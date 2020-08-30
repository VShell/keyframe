use http_types::{Body, Request, Response, StatusCode, Url};
use serde::Serialize;
use smol::Async;
use std::net::{TcpStream, ToSocketAddrs};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid url")]
    InvalidUrl,
    #[error("unknown host")]
    UnknownHost,
    #[error("connecting to server: {0}")]
    Connection(#[source] std::io::Error),
    #[error("initialising TLS: {0}")]
    Tls(#[source] async_native_tls::Error),
    #[error("sending/receiving HTTP: {0}")]
    Http(http_types::Error),
    #[error("bad status code: {0}")]
    StatusCode(StatusCode)
}

#[derive(Serialize)]
struct NotifyBody<'a> {
    online: bool,
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mpd_url: Option<&'a str>,
}

async fn fetch(req: Request) -> Result<Response, Error> {
    let host = req.url().host_str().ok_or(Error::InvalidUrl)?;
    let port = req.url().port_or_known_default().ok_or(Error::InvalidUrl)?;

    let addr = (host, port).to_socket_addrs().map_err(Error::Connection)?.next().ok_or(Error::UnknownHost)?;
    let stream = Async::<TcpStream>::connect(addr).await.map_err(Error::Connection)?;

    Ok(match req.url().scheme() {
        "http" => async_h1::connect(stream, req).await.map_err(Error::Http)?,
        "https" => {
            // In case of HTTPS, establish a secure TLS connection first.
            let stream = async_native_tls::connect(host, stream).await.map_err(Error::Tls)?;
            async_h1::connect(stream, req).await.map_err(Error::Http)?
        }
        scheme => return Err(Error::InvalidUrl),
    })
}

async fn notify(notify_url: Url, body: &NotifyBody<'_>) -> Result<(), Error> {
    let mut req = Request::post(notify_url);
    req.set_body(Body::from_json(body).unwrap());
    let resp = fetch(req).await?;
    if resp.status() != StatusCode::Ok {
        return Err(Error::StatusCode(resp.status()));
    }
    Ok(())
}

pub async fn notify_online(notify_url: Url, token: &str, mpd_url: &str) -> Result<(), Error> {
    notify(notify_url, &NotifyBody {
        online: true,
        token: token,
        mpd_url: Some(mpd_url),
    }).await
}

pub async fn notify_offline(notify_url: Url, token: &str) -> Result<(), Error> {
    notify(notify_url, &NotifyBody {
        online: false,
        token: token,
        mpd_url: None,
    }).await
}
