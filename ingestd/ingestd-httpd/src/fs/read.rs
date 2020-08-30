use futures::prelude::*;
use iou::SubmissionQueueEvent;
use ringbahn::Cancellation;
use ringbahn::{Drive, Event, Submission};
use std::mem::ManuallyDrop;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;

pub fn alloc_read_buffer() -> Box<[u8]> {
    unsafe { Box::from_raw(std::alloc::alloc_zeroed(std::alloc::Layout::new::<[u8; 4096]>()) as *mut [u8; 4096]) }
}

pub fn read_from_file<D: Drive + Unpin, IO: AsRawFd>(driver: D, io: Rc<IO>, offset: u64, buf: Box<[u8]>, len: usize) -> impl Future<Output=(Box<[u8]>, std::io::Result<usize>)> + Unpin {
    let event = ReadOffset {
        inner: Box::new(ReadOffsetInner {
            io,
            buf,
        }),
        offset,
        len,
    };
    Submission::new(event, driver).map(|(event, res)| {
        (event.inner.buf, res)
    })
}

struct ReadOffsetInner<IO> {
    io: Rc<IO>,
    buf: Box<[u8]>,
}

struct ReadOffset<IO> {
    inner: Box<ReadOffsetInner<IO>>,
    offset: u64,
    len: usize,
}

impl<IO: AsRawFd> Event for ReadOffset<IO> {
    unsafe fn prepare(&mut self, sqe: &mut SubmissionQueueEvent) {
        sqe.prep_read(self.inner.io.as_raw_fd(), &mut self.inner.buf[..self.len], self.offset as usize);
    }

    unsafe fn cancel(this: &mut ManuallyDrop<Self>) -> Cancellation {
        let consumer = Box::into_raw(ManuallyDrop::take(this).inner);
        Cancellation::new(consumer as *mut (), 0, cancel::<IO>)
    }
}

unsafe fn cancel<IO>(consumer: *mut (), _metadata: usize) {
    drop(Box::from_raw(consumer as *mut ReadOffsetInner<IO>));
}
