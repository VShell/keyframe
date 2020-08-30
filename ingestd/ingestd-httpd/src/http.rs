use async_dup::Arc;
use futures::prelude::*;
use http_types::headers::CONNECTION;
use http_types::{Request, Response, StatusCode};
use smol::Async;
use thiserror::Error;

use async_h1::server::Encoder;
pub use async_h1::server::decode;

use crate::tee::Tee;

#[derive(Error, Debug)]
#[error("writing response: {0:?}")]
pub struct ResponseWriterError(#[source] pub Option<Box<dyn std::error::Error + Send + Sync>>);

#[must_use = "this `ResponseWriter` must have a response written to it"]
pub struct ResponseWriter<T> {
    stream: Arc<Async<T>>,
}

impl<T> ResponseWriter<T> where T: 'static, for<'a> &'a T: std::io::Write {
    pub fn new(stream: Arc<Async<T>>) -> ResponseWriter<T> {
        ResponseWriter {
            stream
        }
    }

    pub async fn write(mut self, mut response: Response, request: Request) -> std::io::Result<()> {
        response.insert_header(CONNECTION, "close");
        let encoder = Encoder::new(response, request.method());
        match futures::io::copy(encoder, &mut self.stream).await {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(e.kind(), ResponseWriterError(e.into_inner()))),
        }
    }
}

pub fn write_500<T: 'static>(request: Request, rw: ResponseWriter<T>) -> impl Future<Output=std::io::Result<()>> where for<'a> &'a T: std::io::Write {
    let mut response = Response::new(StatusCode::InternalServerError);
    response.set_body("An internal server error occurred.");
    rw.write(response, request)
}
