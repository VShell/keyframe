#![feature(maybe_uninit_ref)]

use anyhow::Result;
use async_dup::Arc;
use http_types::{Method, Request};
use futures::channel::{mpsc, oneshot};
use futures::future::{Future, pending};
use serde::Deserialize;
use smol::{Async, Task};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::FromRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::rc::Rc;
use thiserror::Error;

mod fs;
mod http;
mod ring;
mod get;
mod put;
mod tee;
use fs::{Driver, get_file_path, spawn_driver};
use http::{ResponseWriter, ResponseWriterError, write_500};
use get::{process_head_request, process_get_request};
use put::{process_put_request, process_delete_request};
use tee::Tee;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
  web_root: PathBuf,
}

pub struct State {
    uring_driver: &'static Driver,
    file_root: PathBuf,
    in_flight_files: RefCell<HashMap<String, mpsc::Sender<(Request, http::ResponseWriter<UnixStream>, oneshot::Sender<Result<()>>)>>>,
}

#[derive(Error, Debug)]
#[error("bad HTTP method")]
struct BadMethod;

async fn process_request<T, F, U>(stream: Async<T>, state: Rc<State>, f: F)
  where for<'a> &'a T: std::io::Write + std::io::Read,
  T: std::os::unix::io::AsRawFd + Send + Sync + 'static,
  F: FnOnce(Request, ResponseWriter<T>, Rc<State>) -> U,
  U: Future<Output=Result<()>>,
{
    /*let (stream, path, fd) = {
        use std::fs::File;
        use std::time::SystemTime;
        use std::os::unix::io::AsRawFd;
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let path = format!("/var/lib/ingestd/httpd/dumps/{}", duration.as_micros());
        let fd = stream.as_raw_fd();
        (Tee::new(stream, File::create(&path).unwrap()), path, fd)
    };*/
    let stream = Arc::new(stream);
    let request = http::decode(stream.clone()).await.unwrap().unwrap();
    /*{
        eprintln!("{} is dumped at {}, fd {}", request.url().path(), path, fd);
    }*/
    let request_clone = request.clone();
    let response_writer = ResponseWriter::new(stream.clone());
    if let Err(err) = f(request, response_writer, state).await {
        eprintln!("something went wrong: {:?}", err);
        for err in err.chain() {
            if err.is::<ResponseWriterError>() {
                eprintln!("is a response writer error");
                return;
            }
        }
        let _ = write_500(request_clone, ResponseWriter::new(stream)).await;
    }
}

async fn process_public_request(request: Request, rw: ResponseWriter<UnixStream>, state: Rc<State>) -> Result<()> {
    match request.method() {
        Method::Head => process_head_request(request, rw, state).await,
        Method::Get => process_get_request(request, rw, state).await,
        _ => Err(BadMethod.into()),
    }
}

async fn process_private_request(request: Request, rw: ResponseWriter<TcpStream>, state: Rc<State>) -> Result<()> {
    match request.method() {
        Method::Put => process_put_request(request, rw, state).await,
        Method::Delete => process_delete_request(request, rw, state).await,
        _ => Err(BadMethod.into()),
    }
}

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        eprintln!("usage: ingestd-httpd config");
        std::process::exit(1);
    }
    let config = {
        let config_filename = args.nth(1).unwrap();
        let config_toml = std::fs::read(config_filename).unwrap();
        toml::from_slice::<Config>(&config_toml).unwrap()
    };

    let state = Rc::new(State {
        uring_driver: spawn_driver(),
        file_root: config.web_root,
        in_flight_files: RefCell::new(HashMap::new()),
    });

    let public_listener = unsafe { UnixListener::from_raw_fd(0) };
    let private_listener = unsafe { TcpListener::from_raw_fd(3) };

    smol::run(async move {
        let public_listener = Async::<UnixListener>::new(public_listener).unwrap();
        let private_listener = Async::<TcpListener>::new(private_listener).unwrap();
        let public_state = state.clone();
        Task::local(async move {
            loop {
                let (stream, _peer_addr) = public_listener.accept().await.unwrap();
                Task::local(process_request(stream, public_state.clone(), process_public_request)).detach();
            }
        }).detach();
        Task::local(async move {
            loop {
                let (stream, _peer_addr) = private_listener.accept().await.unwrap();
                Task::local(process_request(stream, state.clone(), process_private_request)).detach();
            }
        }).detach();
        pending::<()>().await
    });
}
