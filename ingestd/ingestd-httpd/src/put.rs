use anyhow::Result;
use futures::prelude::*;
use futures::channel::mpsc;
use futures::{pin_mut, select};
use http_types::{Request, Response, StatusCode};
use scopeguard::guard;
use std::fs::{File, OpenOptions};
use std::net::TcpStream;
use std::rc::Rc;

use crate::State;
use crate::fs::{get_file_path, write_from_ring};
use crate::get::process_get_request_from_ring;
use crate::http::ResponseWriter;
use crate::ring::{Ring, RingConsumer};

async fn write_file<'a>(state: &'a State, consumer: RingConsumer<'a>, file: Rc<File>, request: Request, rw: ResponseWriter<TcpStream>) -> Result<()> {
    //futures::io::copy_buf(reader, &mut futures::io::sink()).await?;
    let _drop_guard = guard((), |()| {
        eprintln!("dropped!");
    });
    eprintln!("writing file");
    write_from_ring(state.uring_driver, file, consumer).await?;
    eprintln!("writing response");
    write_201(request, rw).await?;
    eprintln!("written response");
    Ok(())
}

pub async fn process_put_request(mut request: Request, rw: ResponseWriter<TcpStream>, state: Rc<State>) -> Result<()> {
    {
        use std::time::SystemTime;
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        eprintln!("POST {} at {}:{}", request.url().path(), duration.as_secs(), duration.subsec_millis());
    }

    let body = request.take_body();
    let path = request.url().path().to_string();

    let mut get_request_receiver = {
        let mut in_flight_files = state.in_flight_files.borrow_mut();
        if in_flight_files.contains_key(&path) {
            drop(in_flight_files);
            write_409(request, rw).await?;
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel(0);
        in_flight_files.insert(path.clone(), sender);
        receiver
    };
    let _remove_guard = guard((), |()| {
        state.in_flight_files.borrow_mut().remove(&path);
    });

    let path = get_file_path(&state, &path)?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let file = Rc::new(OpenOptions::new().create(true).truncate(true).write(true).read(true).open(path)?);

    let mut ring_state = Ring::state();
    let ring = ring_state.ring(body, |r| write_file(&*state, r, file.clone(), request, rw));
    pin_mut!(ring);
    loop {
        select! {
            (request, stream, result_sender) = get_request_receiver.select_next_some() => {
                ring.as_mut().push(|ring_reader| async {
                    let result = process_get_request_from_ring(request, stream, &*state, ring_reader, file.clone()).await;
                    // We don't care whether the other side actually picks up this error
                    let _ = result_sender.send(result);
                });
            },
            res = ring.as_mut() => {
                res?;
                return Ok(());
            },
        }
    }
}

pub async fn process_delete_request(mut request: Request, rw: ResponseWriter<TcpStream>, state: Rc<State>) -> Result<()> {
  let path = request.url().path().to_string();
  std::fs::remove_file(get_file_path(&state, &path)?)?;
  write_201(request, rw).await?;
  eprintln!("finished delete");
  Ok(())
}

fn write_409(request: Request, rw: ResponseWriter<TcpStream>) -> impl Future<Output=std::io::Result<()>> {
    let mut response = Response::new(StatusCode::Conflict);
    response.set_body("This file is already being uploaded.");
    rw.write(response, request)
}

fn write_201(request: Request, rw: ResponseWriter<TcpStream>) -> impl Future<Output=std::io::Result<()>> {
    rw.write(Response::new(StatusCode::Created), request)
}
