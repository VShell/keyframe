use futures::prelude::*;
use futures::future::poll_fn;
use futures::task::{AtomicWaker, Poll};
use futures::{pin_mut, select};
use http_types::Url;
use pathsearch::find_executable_in_path;
use serde::Serialize;
use smol::{Async, Task, Timer};
use sqlx::{SqliteConnection, query};
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use crate::log::Logger;
use crate::notify::{notify_online, notify_offline};
use crate::pidfd::Pidfd;
use crate::shared::{Connection, NewConnection};

fn spawn(gpac_path: &CStr, argv: Vec<CString>, logger: &Logger) -> std::io::Result<(Pidfd, UnixStream)> {
    use libc::*;
    use crate::syscall::{CloneArgs, clone3};

    eprintln!("{:?}", argv);
    let argv = std::iter::once(gpac_path.as_ptr()).chain(argv.iter().map(|s| s.as_ptr())).chain(std::iter::once(std::ptr::null())).collect::<Vec<_>>();

    let (sender, receiver) = UnixStream::pair()?;

    let mut clone_args = CloneArgs::default();
    let mut pidfd: RawFd = 0;
    clone_args.flags = 0x100001000; // CLONE_CLEAR_SIGHAND | CLONE_PIDFD
    clone_args.pidfd = &mut pidfd as *mut RawFd as u64;

    unsafe {
        let sndbuf = 5000 as c_int;
        let rcvbuf = 5000 as c_int;

        let gpac_pid = clone3(&clone_args);
        if gpac_pid == -1 {
            eprintln!("blargh?");
            Err(std::io::Error::last_os_error())
        } else if gpac_pid == 0 {
            // Child process - we can only call async-signal-safe functions and *must not panic*
            if close(0) == -1 {
                let error_message = "close(0) failed";
                write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                _exit(1);
            }
            if open(b"/dev/null\0" as *const [u8] as *const c_char, O_RDONLY) != 0 {
                let error_message = "open(/dev/null) failed";
                write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                _exit(1);
            }
            if dup2(logger.as_raw_fd(), 1) == -1 {
                let error_message = "dup2(logger, stdout) failed";
                write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                _exit(1);
            }
            if dup2(1, 2) == -1 {
                let error_message = "dup2(logger, stderr) failed";
                write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                _exit(1);
            }
            if receiver.as_raw_fd() == 3 {
                if fcntl(3, F_SETFD, fcntl(3, F_GETFD) & !FD_CLOEXEC) == -1 {
                    let error_message = "fcntl(3, F_SETFD, !FD_CLOEXEC) failed";
                    write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                    _exit(1);
                }
            } else {
                if fcntl(3, F_GETFD) != -1 || std::ptr::read(__errno_location()) != EBADF {
                    if close(3) == -1 {
                        let error_message = "close(3) failed";
                        write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                        _exit(1);
                    }
                }
                if dup2(receiver.as_raw_fd(), 3) == -1 {
                    let error_message = "dup2(receiver, 3) failed";
                    write(logger.as_raw_fd(), error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
                    _exit(1);
                }
            }
            execv(gpac_path.as_ptr(), &*argv as *const [*const c_char] as *const *const c_char);

            let error_message = "Error execing gpac";
            write(2, error_message.as_bytes() as *const [u8] as *const c_void, error_message.len());
            _exit(127);
        } else {
            // Parent process
            eprintln!("pidfd: {}", pidfd);
            Ok((Pidfd(pidfd), sender))
        }
    }
}

async fn handle_gpac_sender(sender: UnixStream, connection: Arc<Connection>) -> std::io::Result<()> {
    let mut sender = Async::new(sender)?;
    let mut packets = Vec::new();
    loop {
        let closed = poll_fn(|cx| {
            eprintln!("waiting for packet");
            connection.gpac_waker.register(cx.waker());
            let mut data = connection.data.lock().unwrap();
            if data.packets.is_empty() && !data.closed {
                Poll::Pending
            } else {
                std::mem::swap(&mut packets, &mut data.packets);
                Poll::Ready(data.closed)
            }
        }).await;

        if closed {
            return Ok(());
        }

        for packet in packets.drain(..) {
            eprintln!("sending packet");
            sender.write_all(&packet.buffer).await?;
            eprintln!("sent packet");
        }
    }
}

async fn kill_gpac(pidfd: &Pidfd, logger: &Logger) -> std::io::Result<()> {
    logger.log("Waiting for gpac to exit");
    Timer::new(Duration::from_secs(5)).await;

    logger.log("Sending SIGTERM");
    pidfd.signal(libc::SIGTERM)?;

    Timer::new(Duration::from_secs(10)).await;

    logger.log("Sending SIGKILL");
    pidfd.signal(libc::SIGKILL)?;

    Ok(())
}

async fn handle_gpac(gpac_path: &CStr, gpac_argv: Vec<CString>, notify_url: String, notify_token: String, stream_uuid: Uuid, external_url: &'static str, logger: Logger, connection: Arc<Connection>) -> std::io::Result<()> {
    let (pidfd, sender) = spawn(gpac_path, gpac_argv, &logger).unwrap();
    let pidfd_guard = pidfd.guard();
    let pidfd_wait = pidfd.wait().fuse();
    pin_mut!(pidfd_wait);

    let notify_url_parsed = Url::parse(&notify_url).unwrap();
    let mpd_url = format!("{}/{}.mpd", external_url.strip_suffix('/').unwrap_or(external_url), stream_uuid);
    let notify_task = Task::spawn({
        let logger = logger.clone();
        let notify_url_parsed = notify_url_parsed.clone();
        async move {
            if let Err(e) = notify_online(notify_url_parsed, &notify_token, &mpd_url).await {
                logger.log(&format!("Online notification to {} failed: {}", notify_url, e));
            }
            notify_token
        }
    });

    let code = select! {
        res = Task::spawn(handle_gpac_sender(sender, connection.clone())).fuse() => {
            if let Err(e) = res {
                logger.log("gpac sender task failed");
            }
            logger.log("Closed due to sender task finishing");
            connection.data.lock().unwrap().closed = true;
            select! {
                res = kill_gpac(&pidfd, &logger).fuse() => {
                    if let Err(e) = res {
                        logger.log("Killing gpac failed");
                    }
                    pidfd_wait.await
                },
                code = pidfd_wait.as_mut() => {
                    code
                },
            }
        },
        code = pidfd_wait.as_mut() => {
            logger.log("Closed due to pidfd");
            connection.data.lock().unwrap().closed = true;
            code
        },
    };
    std::mem::forget(pidfd_guard);

    let notify_url = notify_task.await;
    if let Err(e) = notify_offline(notify_url_parsed, &notify_url).await {
        logger.log(&format!("Offline notification to {} failed: {}", notify_url, e));
    }

    logger.log(&format!("code: {}", code.unwrap().si_errno));
    Ok(())
}

fn get_gpac_argv(httpd_url: &str, stream_uuid: &Uuid) -> Vec<CString> {
    let uuid_hyphenated = stream_uuid.to_hyphenated_ref();
    let mpd_url = format!("{}/{}.mpd", httpd_url.strip_suffix('/').unwrap_or(httpd_url), stream_uuid);
    vec![
        CString::new("-log-utc").unwrap(),
        CString::new("-logs=all@info").unwrap(),
        CString::new(format!("src=tcpu://inherit:#Filename={uuid}", uuid=uuid_hyphenated)).unwrap(),
        CString::new(format!("dst={mpd_url}:gpac:template={uuid}_$RepresentationID$$FS$_$Init=init$$Number%05d$:utcs=inband:segext=mp4:hmode=push:profile=live:dmode=dynamic:muxtype=mp4:tfdt_traf:segdur=8:cdur=0.1:asto=7.9:buf=1000", mpd_url=mpd_url, uuid=uuid_hyphenated)).unwrap(),
    ]
}

pub fn listen(waker: Arc<AtomicWaker>, new_connections: Arc<Mutex<Vec<NewConnection>>>, mut db: SqliteConnection, httpd_url: String, external_url: String) -> impl Future<Output=()> {
    let gpac_path = find_executable_in_path("gpac").unwrap().into_os_string();
    let gpac_path: &'static CStr = Box::leak(CString::new(gpac_path.into_vec()).unwrap().into_boxed_c_str());

    let external_url: &'static str = Box::leak(external_url.into_boxed_str());

    async move {
        loop {
            let new_connections = poll_fn(|cx| {
                waker.register(cx.waker());
                let new_connections = std::mem::replace(&mut *new_connections.lock().unwrap(), Vec::new());
                if new_connections.len() > 0 {
                    Poll::Ready(new_connections)
                } else {
                    Poll::Pending
                }
            }).await;
            for connection in new_connections {
                eprintln!("spawning");
                let NewConnection { mut logger, stream_id, stream_uuid, connection } = connection;
                logger.set_prefix("[ingestd-srt::gpac] ");
                let stream_id = match std::str::from_utf8(&stream_id) {
                    Ok(s) => s,
                    Err(_) => { eprintln!("couldn't parse C string: {:?}", &stream_id); continue},
                };
                if stream_id.len() >= 6 && &stream_id[0..6] != "#!::u=" {
                    continue;
                }
                let stream_userid = match stream_id[6..].parse::<u32>() {
                    Ok(u) => u,
                    Err(_) => continue,
                };
                eprintln!("spawning stream {}", stream_userid);
                match query!("SELECT notify_url, token FROM streams where id = ?", stream_userid as i32).fetch_one(&mut db).await {
                    Ok(stream_row) => {
                        let gpac_argv = get_gpac_argv(&httpd_url, &stream_uuid);
                        Task::spawn(async move {
                            handle_gpac(gpac_path, gpac_argv, stream_row.notify_url, stream_row.token, stream_uuid, &external_url, logger, connection).await.unwrap()
                        }).detach()
                    },
                    Err(e) => connection.data.lock().unwrap().closed = true,
                }
            }
        }
    }
}
