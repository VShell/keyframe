use arc_swap::ArcSwap;
use thiserror::Error;
use libsrt_sys::*;
use futures::task::AtomicWaker;
use openat::Dir;
use roaring::RoaringBitmap;
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::net::UdpSocket;
use std::os::unix::io::IntoRawFd;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::log::Logger;
use crate::shared::{Connection, NewConnection, Packet};

struct ShutdownGuard;

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        unsafe { srt_cleanup(); }
    }
}

struct EpollReleaseGuard(libc::c_int);

impl Drop for EpollReleaseGuard {
    fn drop(&mut self) {
        unsafe { srt_epoll_release(self.0); }
    }
}

#[derive(Error, Debug)]
#[error("{}", unsafe { CStr::from_ptr(srt_strerror(self.0, 0)) }.to_string_lossy())]
pub struct SrtError(SRT_ERRNO);

fn srt(res: libc::c_int) -> Result<libc::c_int, SrtError> {
    if res != -1 {
        Ok(res)
    } else {
        Err(SrtError(unsafe { srt_getlasterror(std::ptr::null_mut()) }))
    }
}

unsafe fn remove_connection(map: &mut HashMap<SRTSOCKET, Arc<Connection>>, epoll: libc::c_int, fd: SRTSOCKET) -> Result<(), SrtError> {
    eprintln!("closed in remove_connection");
    srt(srt_epoll_remove_usock(epoll, fd))?;
    let connection = map.remove(&fd).unwrap();
    connection.data.lock().unwrap().closed = true;
    connection.gpac_waker.wake();
    Ok(())
}

unsafe fn process_socket(fd: SRTSOCKET, connection: &Connection) -> bool {
    let mut data = connection.data.lock().unwrap();

    if data.closed {
        eprintln!("closed");
        return false;
    }

    eprintln!("backlog: {}", data.packets.len());
    for _ in 0..1024-data.packets.len() {
        let mut buffer = vec![0; SRT_LIVE_MAX_PLSIZE as usize];

        let mut msg_ctrl = MaybeUninit::<SRT_MSGCTRL>::uninit();
        srt_msgctrl_init(msg_ctrl.as_mut_ptr());
        let mut msg_ctrl = msg_ctrl.assume_init();

        let len = match srt(srt_recvmsg2(fd, buffer.as_mut_ptr() as *mut libc::c_char, buffer.len() as i32, &mut msg_ctrl as *mut SRT_MSGCTRL)) {
            Err(e) if e.0 == SRT_EASYNCRCV => continue,
            Err(s) => {
                eprintln!("{}", s);
                return false;
            },
            Ok(len) => len,
        };

        if len == 0 {
            eprintln!("0-length message");
            return false;
        }

        buffer.truncate(len as usize);

        data.packets.push(Packet {
            buffer,
            received: Instant::now(),
        });
    }

    true
}

unsafe fn listen(epoll: libc::c_int, listener: SRTSOCKET, log_dir: Dir, gpac_waker: Arc<AtomicWaker>, new_connections: Arc<Mutex<Vec<NewConnection>>>) -> Result<(), SrtError> {
    let mut connections = HashMap::new();
    loop {
        let mut read_fds = [0; 256];
        let mut read_fds_size = 256;

        eprintln!("polling");
        srt(srt_epoll_wait(epoll,
            read_fds.as_mut_ptr(), &mut read_fds_size as *mut libc::c_int,
            std::ptr::null_mut(), std::ptr::null_mut(),
            -1,
            std::ptr::null_mut(), std::ptr::null_mut(),
            std::ptr::null_mut(), std::ptr::null_mut(),
        ))?;

        for i in 0..read_fds_size as usize {
            let fd = read_fds[i];
            if fd == listener {
                let mut addr = MaybeUninit::<libc::sockaddr_in>::uninit();
                let mut addr_size = std::mem::size_of_val(&addr) as libc::c_int;
                let fd = match srt(srt_accept(listener, addr.as_mut_ptr() as *mut libc::sockaddr, &mut addr_size as *mut libc::c_int)) {
                    Err(e) if e.0 == SRT_EASYNCRCV => continue,
                    Err(e) => return Err(e.into()),
                    Ok(fd) => fd,
                };

                eprintln!("accepted fd: {}", fd);

                let epoll_flags = (SRT_EPOLL_IN|SRT_EPOLL_ERR) as SRT_EPOLL_T;
                srt(srt_epoll_add_usock(epoll, fd, &epoll_flags as *const SRT_EPOLL_T))?;

                let connection = Arc::new(Connection::default());
                connections.insert(fd, connection.clone());

                let stream_uuid = Uuid::new_v4();
                let logger = {
                    let mut filename_buf = [0; uuid::adapter::HyphenatedRef::LENGTH];
                    let filename = stream_uuid.to_hyphenated_ref().encode_lower(&mut filename_buf);
                    let log_file = log_dir.append_file(filename as &str, 0o640).unwrap();
                    Logger::from(log_file)
                };

                let stream_id = {
                    let mut stream_id = [0u8; 512];
                    let mut stream_id_size = std::mem::size_of_val(&stream_id) as libc::c_int;
                    srt(srt_getsockflag(fd, SRTO_STREAMID, stream_id.as_mut_ptr() as *mut libc::c_void, &mut stream_id_size as *mut libc::c_int))?;
                    eprintln!("stream id: {:?}", &stream_id[0..32]);
                    Box::from(&stream_id[..stream_id_size as usize])
                };

                new_connections.lock().unwrap().push(NewConnection {
                    logger,
                    stream_id,
                    stream_uuid,
                    connection,
                });
                gpac_waker.wake();
            } else {
                match srt_getsockstate(fd) {
                    SRTS_BROKEN => {
                        remove_connection(&mut connections, epoll, fd)?;
                        continue;
                    },
                    _ => {},
                }

                let connection = &connections[&fd];
                if process_socket(fd, connection) {
                    connection.gpac_waker.wake();
                } else {
                    remove_connection(&mut connections, epoll, fd)?;
                    srt(srt_close(fd))?;
                }
            }
        }
    }
}

struct AuthUserData {
    global_secret: [u8; 32],
    valid_stream_ids: Arc<ArcSwap<RoaringBitmap>>,
}

unsafe extern "C" fn auth(userdata: *mut libc::c_void, sock: SRTSOCKET, _hsversion: libc::c_int, _peeraddr: *const libc::sockaddr, stream_id: *const libc::c_char) -> libc::c_int {
    let userdata = &*(userdata as *const AuthUserData);

    eprintln!("authing");

    let stream_id = match CStr::from_ptr(stream_id).to_str() {
        Ok(s) => s,
        Err(_) => {eprintln!("bad streamid"); return -1},
    };
    if stream_id.len() >= 6 && &stream_id[0..6] != "#!::u=" {
        eprintln!("bad start");
        return -1;
    }

    let stream_userid = match stream_id[6..].parse::<u32>() {
        Ok(u) => u,
        Err(_) => {
            eprintln!("couldn't parse user id");
            return -1;
        },
    };

    if !userdata.valid_stream_ids.load().contains(stream_userid) {
        eprintln!("stream id invalid");
        return -1;
    }

    // Perform a keyed hash of stream_id to get the passphrase
    let hash = blake3::keyed_hash(&userdata.global_secret, stream_id.as_bytes());
    let hash_hex = hash.to_hex();
    srt_setsockflag(sock, SRTO_PASSPHRASE, hash_hex.as_ptr() as *const libc::c_void, hash_hex.len() as libc::c_int);
    eprintln!("hash: {}", hash_hex);
    eprintln!("authed");
    return 0;
}

pub fn spawn_listen(listener_sock: UdpSocket, log_dir: Dir, global_secret: [u8; 32], valid_stream_ids: Arc<ArcSwap<RoaringBitmap>>, gpac_waker: Arc<AtomicWaker>, new_connections: Arc<Mutex<Vec<NewConnection>>>) -> Result<(), SrtError> {
    unsafe {
        srt_setloglevel(7);

        srt(srt_startup())?;
        let guard = ShutdownGuard;

        let listener = srt(srt_create_socket())?;

        let auth_user_data = Box::leak(Box::new(AuthUserData {
            global_secret,
            valid_stream_ids,
        }));
        srt(srt_listen_callback(listener, Some(auth), auth_user_data as *mut AuthUserData as *mut libc::c_void))?;

        let no = false;
        srt(srt_setsockflag(listener, SRTO_RCVSYN, &no as *const bool as *const libc::c_void, std::mem::size_of_val(&no) as libc::c_int))?;
        //let yes = 1 as libc::c_int;
        //srt(srt_setsockflag(listener, SRTO_GROUPCONNECT, &yes as *const libc::c_int as *const libc::c_void, std::mem::size_of_val(&yes) as libc::c_int))?;
        let lossmaxttl = 10 as libc::c_int;
        srt(srt_setsockflag(listener, SRTO_LOSSMAXTTL, &lossmaxttl as *const libc::c_int as *const libc::c_void, std::mem::size_of_val(&lossmaxttl) as libc::c_int))?;

        srt(srt_bind_peerof(listener, listener_sock.into_raw_fd()))?;
        srt(srt_listen(listener, 10))?;

        let epoll = srt(srt_epoll_create())?;
        let listener_epoll_flags = (SRT_EPOLL_IN|SRT_EPOLL_ERR) as SRT_EPOLL_T;
        srt(srt_epoll_add_usock(epoll, listener, &listener_epoll_flags as *const SRT_EPOLL_T))?;

        std::thread::Builder::new().name("srt".to_string()).spawn(move || {
            let _guard = guard;
            if let Err(e) = listen(epoll, listener, log_dir, gpac_waker, new_connections) {
                eprintln!("{}", e);
            }
        }).unwrap();

        Ok(())
    }
}
