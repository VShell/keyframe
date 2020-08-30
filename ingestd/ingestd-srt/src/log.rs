use std::fs::File;
use std::io::Write;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;

#[derive(Clone)]
pub struct Logger {
    file: Arc<File>,
    prefix: &'static str,
}

impl Logger {
    pub fn set_prefix(&mut self, prefix: &'static str) {
        self.prefix = prefix;
    }

    pub fn log(&self, msg: &str) {
        let mut message_buf: [u8; 1024] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        message_buf[..self.prefix.len()].copy_from_slice(self.prefix.as_bytes());
        message_buf[self.prefix.len()..self.prefix.len()+msg.len()].copy_from_slice(msg.as_bytes());
        let _ = (&*self.file).write_all(&message_buf[..self.prefix.len()+msg.len()]);
    }
}

impl AsRawFd for Logger {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl From<File> for Logger {
    fn from(file: File) -> Logger {
        Logger {
            file: Arc::new(file),
            prefix: "",
        }
    }
}
