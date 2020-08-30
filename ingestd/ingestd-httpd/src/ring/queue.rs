use futures::prelude::*;
use futures::task::{Waker, Context, Poll, noop_waker};
use futures::ready;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

struct MagicBuffer {
    data: *mut u8,
    len: usize,
}

fn magic(len: usize) -> nix::Result<MagicBuffer> {
    use std::ffi::CStr;
    use nix::sys::memfd::*;
    use nix::sys::mman::*;
    use nix::unistd::*;

    let fd = memfd_create(unsafe { CStr::from_bytes_with_nul_unchecked(b"magic\0") }, MemFdCreateFlag::empty())?;
    ftruncate(fd, len as i64)?;
    let buffer = unsafe { mmap(std::ptr::null_mut(), len*2, ProtFlags::PROT_NONE, MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS, -1, 0)? };
    unsafe {
        mmap(buffer, len, ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, MapFlags::MAP_SHARED | MapFlags::MAP_FIXED, fd, 0)?;
        mmap(buffer.add(len), len, ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, MapFlags::MAP_SHARED | MapFlags::MAP_FIXED, fd, 0)?;
    }
    close(fd)?;
    Ok(MagicBuffer {
        data: buffer.cast(),
        len,
    })
}

impl Drop for MagicBuffer {
    fn drop(&mut self) {
        use nix::sys::mman::munmap;
        let _ = unsafe { munmap(self.data.cast(), self.len*2) };
    }
}

struct Queue {
    buffer: MagicBuffer,
    write_start: u64,
    write_end: u64,
    write_waker: Waker,
    read_waker: Waker,
}

pub fn queue(capacity: usize) -> (QueueWriter, QueueReader) {
    let queue = Rc::new(RefCell::new(Queue {
        buffer: magic(capacity).unwrap(),
        write_start: 0,
        write_end: capacity as u64,
        write_waker: noop_waker(),
        read_waker: noop_waker(),
    }));
    (QueueWriter { queue: queue.clone() }, QueueReader { queue })
}

pub struct QueueWriter {
    queue: Rc<RefCell<Queue>>,
}

impl QueueWriter {
    pub fn poll_buf(&mut self, cx: &mut Context) -> Poll<Option<&mut [u8]>> {
        let mut queue = self.queue.borrow_mut();

        if queue.write_end == u64::MAX {
            return Poll::Ready(None);
        }

        let len = (queue.write_end-queue.write_start) as usize;
        if len == 0 {
            queue.write_waker = cx.waker().clone();
            return Poll::Pending;
        }

        Poll::Ready(Some(unsafe { std::slice::from_raw_parts_mut(queue.buffer.data.add((queue.write_start % queue.buffer.len as u64) as usize), len) }))
    }

    pub fn written(&self, amount: usize) {
        let mut queue = self.queue.borrow_mut();
        assert!(amount <= (queue.write_end-queue.write_start) as usize);
        queue.write_start += amount as u64;
        std::mem::replace(&mut queue.read_waker, noop_waker()).wake();
    }

    pub fn buffer_from(&self, index: usize) -> &[u8] {
        let queue = self.queue.borrow();
        assert!(index < queue.buffer.len);
        unsafe { std::slice::from_raw_parts(queue.buffer.data.add(index), queue.buffer.len) }
    }
}

impl Drop for QueueWriter {
    fn drop(&mut self) {
        let mut queue = self.queue.borrow_mut();
        queue.write_start = u64::MAX;
    }
}

pub struct QueueReader {
    queue: Rc<RefCell<Queue>>,
}

impl AsyncRead for QueueReader {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let mut this = &mut *self;
        let buffer = ready!(Pin::new(&mut this).poll_fill_buf(cx)?);
        let len = std::cmp::min(buffer.len(), buf.len());
        buf[..len].copy_from_slice(&buffer[..len]);
        Pin::new(this).consume(len);
        Poll::Ready(Ok(len))
    }
}

impl AsyncBufRead for QueueReader {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<std::io::Result<&[u8]>> {
        let mut queue = self.queue.borrow_mut();

        if queue.write_start == u64::MAX {
            return Poll::Ready(Ok(&[]));
        }

        let len = queue.buffer.len-(queue.write_end-queue.write_start) as usize;
        if len == 0 {
            queue.read_waker = cx.waker().clone();
            return Poll::Pending;
        }

        Poll::Ready(Ok(unsafe { std::slice::from_raw_parts(queue.buffer.data.add((queue.write_end % queue.buffer.len as u64) as usize), len) }))
    }

    fn consume(self: Pin<&mut Self>, amount: usize) {
        let mut queue = self.queue.borrow_mut();
        assert!(amount <= queue.buffer.len-(queue.write_end-queue.write_start) as usize);
        queue.write_end += amount as u64;
        std::mem::replace(&mut queue.write_waker, noop_waker()).wake();
    }
}

impl Drop for QueueReader {
    fn drop(&mut self) {
        let mut queue = self.queue.borrow_mut();
        queue.write_end = u64::MAX;
    }
}
