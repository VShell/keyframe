use anyhow::Result;
use futures::prelude::*;
use futures::future::FusedFuture;
use futures::task::{Context, Poll, Waker};
use futures::ready;
use slab::Slab;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::pin::Pin;
use unicycle::FuturesUnordered;

mod queue;
use queue::{QueueReader, QueueWriter, queue};

const RING_SIZE: usize = 1024*512;

struct RingStateInner {
    queue: QueueWriter,
    bytes_written: u64,
    reader_error: Option<Option<std::io::Error>>,
    wakers: Slab<Option<Waker>>,
}

pub struct RingState(MaybeUninit<RefCell<RingStateInner>>);

pub struct Ring<'a, R, CF, F> {
    state: &'a RingState,
    consumer: Result<CF, Option<anyhow::Error>>,
    futures: FuturesUnordered<F>,
    reader: Option<R>,
}

impl RingState {
    pub fn ring<'a, R, CF, F>(&'a mut self, reader: R, make_consumer: impl FnOnce(RingConsumer<'a>) -> CF) -> Ring<'a, R, CF, F> {
        let (queue_writer, queue_reader) = queue(RING_SIZE);
        unsafe {
            self.0.as_mut_ptr().write(RefCell::new(RingStateInner {
                queue: queue_writer,
                bytes_written: 0,
                reader_error: None,
                wakers: Slab::new(),
            }));
        }
        let ring_consumer = RingConsumer {
            state: &*self,
            reader: queue_reader,
        };
        Ring {
            state: &*self,
            consumer: Ok(make_consumer(ring_consumer)),
            futures: FuturesUnordered::new(),
            reader: Some(reader),
        }
    }
}

impl<'a> Ring<'a, (), (), ()> {
    pub fn state() -> RingState {
        RingState(MaybeUninit::uninit())
    }
}

impl<'a, R, CF, F> Ring<'a, R, CF, F> where
    F: Future<Output=()> + 'a
{
    pub fn push(self: Pin<&mut Self>, make_future: impl FnOnce(RingReader<'a>) -> F) {
        // Safety: we do not move data from this
        let this = unsafe { self.get_unchecked_mut() };
        let waker_id = {
            // Safety: we can only be created via RingState::ring(), which initialises the RingState
            let mut state = unsafe { this.state.0.get_ref() }.borrow_mut();
            state.wakers.insert(None)
        };
        this.futures.push(make_future(RingReader {
            state: this.state,
            bytes_read: 0,
            waker_id,
        }));
    }
}

impl<'a, R, CF, F> Future for Ring<'a, R, CF, F> where
    R: AsyncRead,
    CF: Future<Output=Result<()>> + 'a,
    F: Future<Output=()> + 'a
{
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        // Safety: we do not move data from this
        let mut this = unsafe { self.get_unchecked_mut() };
        while let Poll::Ready(Some(())) = Pin::new(&mut this.futures).poll_next(cx) {
        }

        // If the consumer returns, we store its response until this.futures terminates
        match this.consumer {
            Ok(ref mut c) => {
                // Safety: we do not move anything this.consumer can reference
                if let Poll::Ready(res) = unsafe { Pin::new_unchecked(c) }.poll(cx) {
                    if this.futures.is_empty() {
                        return Poll::Ready(res);
                    } else {
                        this.consumer = Err(res.err());
                        this.reader = None;
                        // Safety: we can only be created via RingState::ring(), which initialises the RingState
                        let mut state = unsafe { this.state.0.get_ref() }.borrow_mut();
                        state.reader_error = Some(None);
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            },
            Err(ref mut opt) => {
                if this.futures.is_empty() {
                    let res = opt.take().map_or(Ok(()), Err);
                    return Poll::Ready(res);
                } else {
                    return Poll::Pending;
                }
            },
        };

        // Safety: we can only be created via RingState::ring(), which initialises the RingState
        let mut state = unsafe { this.state.0.get_ref() }.borrow_mut();
        if let Some(ref mut reader) = this.reader {
            let buffer = match ready!(state.queue.poll_buf(cx)) {
                Some(buffer) => buffer,
                None => {
                    // The consumer dropped the QueueReader
                    this.reader = None;
                    state.reader_error = Some(None);
                    return Poll::Pending;
                },
            };
            // Safety: we do not move anything this.reader can reference
            match ready!(unsafe { Pin::new_unchecked(reader) }.poll_read(cx, buffer)) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        // The incoming stream finished
    {
        use std::time::SystemTime;
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        eprintln!("POST finished at {}:{}", duration.as_secs(), duration.subsec_millis());
        eprintln!("buffer length: {}", buffer.len());
    }
                        this.reader = None;
                        state.reader_error = Some(None);
                        // Wake up the RingConsumer
                        state.queue.written(0);
                    } else {
                        state.queue.written(bytes_read);
                        state.bytes_written += bytes_read as u64;
                    }
                },
                Err(e) => {
                    this.reader = None;
                    state.reader_error = Some(Some(e));
                    // Wake up the RingConsumer
                    state.queue.written(0);
                },
            }
            for (_, waker) in state.wakers.iter_mut() {
                waker.take().map(|w| w.wake());
            }
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }
}

impl<'a, R, CF, F> FusedFuture for Ring<'a, R, CF, F> where
    R: AsyncRead,
    CF: Future<Output=Result<()>> + 'a,
    F: Future<Output=()> + 'a
{
    fn is_terminated(&self) -> bool {
        return self.futures.is_empty() && self.consumer.is_err();
    }
}

pub struct RingConsumer<'a> {
    state: &'a RingState,
    reader: QueueReader,
}

impl<'a> AsyncRead for RingConsumer<'a> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let mut this = &mut *self;
        let buffer = ready!(Pin::new(&mut this).poll_fill_buf(cx)?);
        let len = std::cmp::min(buffer.len(), buf.len());
        buf[..len].copy_from_slice(&buffer[..len]);
        Pin::new(this).consume(len);
        Poll::Ready(Ok(len))
    }
}

impl<'a> AsyncBufRead for RingConsumer<'a> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<std::io::Result<&[u8]>> {
        let this = Pin::into_inner(self);
        match Pin::new(&mut this.reader).poll_fill_buf(cx) {
            Poll::Ready(res) => Poll::Ready(res),
            Poll::Pending => {
                // Safety: we can only be created via RingState::ring(), which initialises the RingState
                let mut state = unsafe { this.state.0.get_ref() }.borrow_mut();
                match state.reader_error {
                    Some(ref mut err_option @ Some(_)) => {
                        let err = err_option.take().unwrap();
                        *err_option = Some(err.kind().into());
                        Poll::Ready(Err(err))
                    },
                    Some(None) => Poll::Ready(Ok(&[])),
                    None => Poll::Pending,
                }
            },
        }
    }

    fn consume(mut self: Pin<&mut Self>, amount: usize) {
        Pin::new(&mut self.reader).consume(amount);
    }
}

#[derive(Debug)]
pub struct Overrun;

impl std::error::Error for Overrun {}
impl std::fmt::Display for Overrun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ring buffer overran!")
    }
}

pub struct RingReader<'a> {
    state: &'a RingState,
    bytes_read: u64,
    waker_id: usize,
}

impl RingReader<'_> {
    pub fn skip(&mut self, amount: u64) {
        self.bytes_read += amount;
    }
}

impl<'a> AsyncRead for RingReader<'a> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let mut this = &mut *self;
        // Safety: we can only be created via RingState::ring(), which initialises the RingState
        let mut state = unsafe { this.state.0.get_ref() }.borrow_mut();
        state.wakers[this.waker_id] = Some(cx.waker().clone());
        if state.bytes_written > this.bytes_read+(RING_SIZE as u64) {
            return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, Overrun)));
        }
        if state.bytes_written > this.bytes_read {
            let len = std::cmp::min((state.bytes_written-this.bytes_read) as usize, buf.len());
            buf[..len].copy_from_slice(&state.queue.buffer_from((this.bytes_read % (RING_SIZE as u64)) as usize)[..len]);
            this.bytes_read += len as u64;
            Poll::Ready(Ok(len))
        } else {
            match &state.reader_error {
                Some(Some(err)) => Poll::Ready(Err(err.kind().into())),
                Some(None) => Poll::Ready(Ok(0)),
                None => {
                    Poll::Pending
                },
            }
        }
    }
}
