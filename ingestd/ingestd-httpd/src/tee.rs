use futures::io::{AsyncRead, AsyncWrite};
use futures::task::{Context, Poll};
use std::fs::File;
use std::io::{Result, Write};
use std::marker::Unpin;
use std::pin::Pin;

pub struct Tee<T>(T, File);

impl<T> Tee<T> {
    pub fn new(inner: T, file: File) -> Tee<T> {
        Tee(inner, file)
    }
}

impl<T> AsyncRead for &Tee<T> where for<'a> &'a T: AsyncRead + Unpin {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
       let res = Pin::new(&mut &self.0).poll_read(cx, buf);
       if let Poll::Ready(Ok(bytes_read)) = res {
           (&self.1).write_all(&buf[..bytes_read]).unwrap();
       }
       res
    }
}

impl<T> AsyncWrite for &Tee<T> where for<'a> &'a T: AsyncWrite + Unpin {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut &self.0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut &self.0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut &self.0).poll_close(cx)
    }
}
