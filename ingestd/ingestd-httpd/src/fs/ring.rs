use futures::future::poll_fn;
use futures::io::AsyncBufRead;
use futures::task::Poll;
use futures::ready;
use iou::SubmissionQueueEvent;
use ringbahn::Cancellation;
use ringbahn::{Drive, Event, Submission};
use std::mem::ManuallyDrop;
use std::os::unix::io::AsRawFd;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use crate::ring::RingConsumer;

pub async fn write_from_ring<'a, D, IO>(mut driver: D, io: Rc<IO>, consumer: RingConsumer<'a>) -> std::io::Result<()> where
    for<'b> &'b mut D: Drive,
    IO: AsRawFd
{
    let mut inner = Box::new(WriteRingBufInner {
        io,
        consumer,
    });

    let mut bytes_written = 0;
    loop {
        let buf_res = poll_fn::<std::io::Result<_>, _>(|cx| {
            let buf = ready!(Pin::new(&mut inner.consumer).poll_fill_buf(cx)?);
            if buf.len() == 0 {
                return Poll::Ready(Ok(None));
            }
            let len = std::cmp::min(4096, buf.len());
            let buf = &buf[..len];
            Poll::Ready(Ok(Some(unsafe { NonNull::new_unchecked(buf as *const [u8] as *mut [u8]) })))
        }).await?;
        if let Some(buf) = buf_res {
            let event = WriteRingBuf {
                inner,
                buf,
                offset: bytes_written,
            };
            let (event, res) = Submission::new(event, &mut driver).await;
            let bytes_consumed = res?;
            inner = event.inner;
            Pin::new(&mut inner.consumer).consume(bytes_consumed);
            bytes_written += bytes_consumed as u64;
        } else {
            eprintln!("wrote {} bytes to file", bytes_written);
            return Ok(());
        }
    }
}

struct WriteRingBufInner<'a, IO> {
    io: Rc<IO>,
    consumer: RingConsumer<'a>,
}

struct WriteRingBuf<'a, IO> {
    inner: Box<WriteRingBufInner<'a, IO>>,
    buf: NonNull<[u8]>,
    offset: u64,
}

impl<'a, IO: AsRawFd> Event for WriteRingBuf<'a, IO> {
    unsafe fn prepare(&mut self, sqe: &mut SubmissionQueueEvent) {
        sqe.prep_write(self.inner.io.as_raw_fd(), self.buf.as_ref(), self.offset as usize);
    }

    unsafe fn cancel(this: &mut ManuallyDrop<Self>) -> Cancellation {
        let consumer = Box::into_raw(ManuallyDrop::take(this).inner);
        Cancellation::new(consumer as *mut (), 0, cancel::<IO>)
    }
}

unsafe fn cancel<IO>(consumer: *mut (), _metadata: usize) {
    drop(Box::from_raw(consumer as *mut WriteRingBufInner<'_, IO>));
}
