use smol::Async;
use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, RawFd};

const P_PIDFD: libc::idtype_t = 3;

pub struct Pidfd(pub RawFd);

impl Pidfd {
    pub fn guard(&self) -> DropGuard {
        DropGuard(self)
    }

    pub fn signal(&self, sig: libc::c_int) -> std::io::Result<()> {
        use crate::syscall::pidfd_send_signal;
        if unsafe { pidfd_send_signal(self.0, sig, std::ptr::null(), 0) } == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub async fn wait(&self) -> std::io::Result<libc::siginfo_t> {
        use libc::*;

        if self.0 == -1 {
            return Err(std::io::Error::from_raw_os_error(EBADF));
        }

        // Need a quick wrapper for Async
        struct FdWrapper(RawFd);
        impl AsRawFd for FdWrapper {
            fn as_raw_fd(&self) -> RawFd {
                self.0
            }
        }

        let mut siginfo = MaybeUninit::<siginfo_t>::uninit();
        let fd_wrapper = Async::new(FdWrapper(self.0))?;
        fd_wrapper.read_with(|wrapper| {
            if unsafe { waitid(P_PIDFD, wrapper.0 as id_t, siginfo.as_mut_ptr(), WEXITED|WNOHANG) } == -1 {
                let err = std::io::Error::last_os_error();
                if let Some(ECHILD) = err.raw_os_error() {
                    Err(std::io::ErrorKind::WouldBlock.into())
                } else {
                    Err(err)
                }
            } else if unsafe { siginfo.get_ref().si_signo } == 0 {
                Err(std::io::ErrorKind::WouldBlock.into())
            } else {
                Ok(())
            }
        }).await?;

        Ok(unsafe { siginfo.assume_init() })
    }
}

impl AsRawFd for Pidfd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Drop for Pidfd {
    fn drop(&mut self) {
        use libc::close;
        unsafe { close(self.0) };
    }
}

pub struct DropGuard<'a>(&'a Pidfd);

impl Drop for DropGuard<'_> {
    fn drop(&mut self) {
        use libc::*;
        if self.0.signal(SIGKILL).is_ok() {
            let mut siginfo = MaybeUninit::<siginfo_t>::uninit();
            unsafe { waitid(P_PIDFD, self.0.as_raw_fd() as id_t, siginfo.as_mut_ptr(), WEXITED) };
        }
    }
}
