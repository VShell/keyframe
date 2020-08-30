use anyhow::{Result, anyhow};
use futures::prelude::*;
use futures::channel::oneshot;
use futures::future::poll_fn;
use futures::task::{Context, Poll};
use futures::ready;
use http_types::headers::ACCESS_CONTROL_ALLOW_ORIGIN;
use http_types::{Body, Request, Response, StatusCode};
use owning_ref::OwningHandle;
use std::fs::{File, Metadata};
use std::os::unix::net::UnixStream;
use std::pin::Pin;
use std::rc::Rc;

use crate::State;
use crate::fs::{alloc_read_buffer, get_file_path, read_from_file};
use crate::http::{ResponseWriter, ResponseWriterError};
use crate::ring::{Overrun, RingReader};

pub async fn process_head_request(request: Request, rw: ResponseWriter<UnixStream>, state: Rc<State>) -> Result<()> {
    if state.in_flight_files.borrow().contains_key(request.url().path()) {
        Ok(write_in_flight_header(request, rw).await?)
    } else {
        match std::fs::metadata(get_file_path(&state, request.url().path()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?) {
            Ok(metadata) => Ok(write_at_rest_header(request, rw, metadata).await?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound || e.kind() == std::io::ErrorKind::PermissionDenied => Ok(write_404(request, rw).await?),
            Err(e) => Err(e.into()),
        }
    }
}

pub async fn process_get_request(request: Request, rw: ResponseWriter<UnixStream>, state: Rc<State>) -> Result<()> {
    {
        use std::time::SystemTime;
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        eprintln!("GET  {} at {}:{}", request.url().path(), duration.as_secs(), duration.subsec_millis());
    }
    let (request, rw) = {
        let in_flight_files = state.in_flight_files.borrow();
        if let Some(in_flight_file) = in_flight_files.get(request.url().path()) {
            let (sender, receiver) = oneshot::channel();
            // .clone().try_send() is guaranteed not to fail unless the receiver is dropped
            match in_flight_file.clone().try_send((request, rw, sender)) {
                Ok(()) => {
                    drop(in_flight_files);
                    return receiver.await.unwrap_or(Err(ResponseWriterError(Some(anyhow!("file sender future dropped").into())).into()));
                },
                Err(e) => {
                    let (request, rw, _) = e.into_inner();
                    (request, rw)
                },
            }
        } else {
            (request, rw)
        }
    };
    match File::open(get_file_path(&state, request.url().path()).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?) {
        Ok(file) => Ok(write_at_rest_response(request, rw, &*state, Rc::new(file)).await?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound || e.kind() == std::io::ErrorKind::PermissionDenied => Ok(write_404(request, rw).await?),
        Err(e) => Err(e.into()),
    }
}

pub async fn process_get_request_from_ring<'a>(request: Request, rw: ResponseWriter<UnixStream>, state: &'a State, ring_reader: RingReader<'a>, file: Rc<File>) -> Result<()> {
    write_in_flight_response(request, rw, state, ring_reader, file).await?;
    {
        use std::time::SystemTime;
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        eprintln!("GET  finished at {}:{}", duration.as_secs(), duration.subsec_millis());
    }
    Ok(())
}

trait GetBodyRead {
    fn read<'a>(&'a mut self, buffer: Box<[u8]>) -> Pin<Box<dyn Future<Output=(Box<[u8]>, std::io::Result<usize>)> + 'a>>;
}

enum GetBody<'a, T> {
    Reading(OwningHandle<&'a mut T, Box<Pin<Box<dyn Future<Output=(Box<[u8]>, std::io::Result<usize>)> + 'a>>>>),
    Buffer(Box<[u8]>, usize, usize, &'a mut T),
    Broken,
}

impl<'a, T: GetBodyRead + 'a> AsyncRead for GetBody<'a, T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let mut this = &mut *self;
        let buffer = ready!(Pin::new(&mut this).poll_fill_buf(cx)?);
        let len = std::cmp::min(buffer.len(), buf.len());
        buf[..len].copy_from_slice(&buffer[..len]);
        Pin::new(this).consume(len);
        Poll::Ready(Ok(len))
    }
}

impl<'a, T: GetBodyRead + 'a> AsyncBufRead for GetBody<'a, T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context) -> Poll<std::io::Result<&[u8]>> {
        let this = Pin::into_inner(self);
        match *this {
            GetBody::Reading(_) => {
                let (buffer, len) = {
                    let read = match *this {
                        GetBody::Reading(ref mut read) => read,
                        _ => unreachable!(),
                    };
                    let (buffer, res) = ready!(Pin::as_mut(&mut **read).poll(cx));
                    let len = res?;
                    (buffer, len)
                };
                let old = std::mem::replace(this, GetBody::Broken);
                let read = match old {
                    GetBody::Reading(read) => read.into_owner(),
                    _ => unreachable!(),
                };
                *this = GetBody::Buffer(buffer, 0, len, read);
                match *this {
                    GetBody::Buffer(ref buffer, _, len, _) => Poll::Ready(Ok(&buffer[..len])),
                    _ => unreachable!(),
                }
            },
            GetBody::Buffer(_, index, len, _) => {
                let buffer = match *this {
                    GetBody::Buffer(ref buffer, _, _, _) => buffer,
                    _ => unreachable!(),
                };
                Poll::Ready(Ok(&buffer[index..len]))
            },
            _ => unreachable!(),
        }
    }

    fn consume(mut self: Pin<&mut Self>, amount: usize) {
        let this = &mut *self;
        let needs_refill = match *this {
            GetBody::Buffer(_, ref mut index, len, _) => {
                *index += amount;
                *index == len
            },
            _ => unreachable!(),
        };
        if needs_refill {
            let old = std::mem::replace(this, GetBody::Broken);
            // Grab buffer and T (being temporarily broken)
            let (buffer, reader) = match old {
                GetBody::Buffer(buffer, _, _, reader) => (buffer, reader),
                _ => unreachable!(),
            };
            *this = GetBody::Reading(OwningHandle::new_with_fn(reader, |reader| {
                Box::new(unsafe { &mut *(reader as *mut T) }.read(buffer))
            }));
        }
    }
}

// Safety: These are *actually* unsound, but it's ok because we never actually send or sync this type
// This is only required because Body requires that its Reader is Send and Sync
unsafe impl<T> Send for GetBody<'_, T> {}
unsafe impl<T> Sync for GetBody<'_, T> {}

struct GetBodyAtRest<'a> {
    state: &'a State,
    bytes_read: u64,
    file: Rc<File>,
}

impl GetBodyRead for GetBodyAtRest<'_> {
    fn read<'a>(&'a mut self, buffer: Box<[u8]>) -> Pin<Box<dyn Future<Output=(Box<[u8]>, std::io::Result<usize>)> + 'a>> {
        Box::pin(async move {
            let to_read = buffer.len();
            let (buffer, res) = read_from_file(self.state.uring_driver, self.file.clone(), self.bytes_read, buffer, to_read).await;
            if let Ok(bytes_read) = res {
                self.bytes_read += bytes_read as u64;
            }
            (buffer, res)
        })
    }
}

struct GetBodyFromRing<'a> {
    state: &'a State,
    bytes_read: u64,
    ring_reader: RingReader<'a>,
    file: Rc<File>,
}

impl GetBodyRead for GetBodyFromRing<'_> {
    fn read<'a>(&'a mut self, mut buffer: Box<[u8]>) -> Pin<Box<dyn Future<Output=(Box<[u8]>, std::io::Result<usize>)> + 'a>> {
        Box::pin(async move {
            match poll_fn(|cx| Pin::new(&mut self.ring_reader).poll_read(cx, &mut buffer)).await {
                Ok(bytes_read) => {
                    self.bytes_read += bytes_read as u64;
                    return (buffer, Ok(bytes_read));
                },
                Err(e) if e.get_ref().filter(|e| e.is::<Overrun>()).is_some() => {},
                Err(e) => {
                    return (buffer, Err(e))
                },
            }

            // Overrun, read from file
            let to_read = buffer.len();
            let (buffer, res) = read_from_file(self.state.uring_driver, self.file.clone(), self.bytes_read, buffer, to_read).await;
            if let Ok(bytes_read) = res {
                self.bytes_read += bytes_read as u64;
                self.ring_reader.skip(bytes_read as u64);
            }
            (buffer, res)
        })
    }
}

async fn write_in_flight_response<'a>(request: Request, rw: ResponseWriter<UnixStream>, state: &State, ring_reader: RingReader<'a>, file: Rc<File>) -> std::io::Result<()> {
    let mut read = GetBodyFromRing {
        state,
        bytes_read: 0,
        ring_reader,
        file,
    };
    let mut body_reader = GetBody::Buffer(alloc_read_buffer(), 0, 0, &mut read);
    Pin::new(&mut body_reader).consume(0);
    // Safety: This is *actually* unsound, but ok because we don't ever use it outside the 'a lifetime
    write_200(request, rw, Body::from_reader(unsafe { std::mem::transmute::<_, GetBody<'static, GetBodyFromRing<'static>>>(body_reader) }, None)).await
}

fn write_in_flight_header(request: Request, rw: ResponseWriter<UnixStream>) -> impl Future<Output=std::io::Result<()>> {
    write_200(request, rw, Body::from_reader(futures::io::empty(), None))
}

async fn write_at_rest_response(request: Request, rw: ResponseWriter<UnixStream>, state: &State, file: Rc<File>) -> std::io::Result<()> {
    let metadata = file.metadata()?;
    let mut read = GetBodyAtRest {
        state,
        bytes_read: 0,
        file,
    };
    let mut body_reader = GetBody::Buffer(alloc_read_buffer(), 0, 0, &mut read);
    Pin::new(&mut body_reader).consume(0);
    // Safety: This is *actually* unsound, but ok because we don't ever use it outside its lifetime
    //write_200(request, rw, Body::from_reader(unsafe { std::mem::transmute::<_, GetBody<'static, GetBodyAtRest<'static>>>(body_reader) }, Some(metadata.len() as usize))).await
    write_200(request, rw, Body::from_reader(unsafe { std::mem::transmute::<_, GetBody<'static, GetBodyAtRest<'static>>>(body_reader) }, None)).await
}

fn write_at_rest_header(request: Request, rw: ResponseWriter<UnixStream>, metadata: Metadata) -> impl Future<Output=std::io::Result<()>> {
    //write_200(request, rw, Body::from_reader(futures::io::empty(), Some(metadata.len() as usize)))
    write_200(request, rw, Body::from_reader(futures::io::empty(), None))
}

fn write_200(request: Request, rw: ResponseWriter<UnixStream>, body: Body) -> impl Future<Output=std::io::Result<()>> {
    let mut response = Response::new(StatusCode::Ok);
    response.insert_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    let path = request.url().path();
    if path.ends_with(".mpd") {
        response.set_content_type("application/dash+xml".parse().unwrap());
    } else if path.ends_with(".mp4") {
        response.set_content_type("video/mp4".parse().unwrap());
    } else if path.ends_with(".html") {
        response.set_content_type("text/html".parse().unwrap());
    }
    response.set_body(body);
    rw.write(response, request)
}

fn write_404(request: Request, rw: ResponseWriter<UnixStream>) -> impl Future<Output=std::io::Result<()>> {
    eprintln!("404");
    let mut response = Response::new(StatusCode::NotFound);
    response.insert_header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    response.set_body("File not found.");
    rw.write(response, request)
}
