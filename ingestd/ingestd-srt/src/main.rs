#![feature(str_strip)]
#![feature(maybe_uninit_ref)]
#![recursion_limit="256"]

use arc_swap::ArcSwap;
use futures::executor::block_on;
use futures::task::AtomicWaker;
use openat::Dir;
use serde::Deserialize;
use sqlx::{Connect, SqliteConnection};
use std::net::UdpSocket;
use std::os::unix::io::FromRawFd;
use std::sync::{Arc, Mutex};

mod gpac;
mod log;
mod notify;
mod pidfd;
mod shared;
mod srt;
mod stream_db;
mod syscall;

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    stream_logs: String,
    #[serde(deserialize_with = "secret_from_toml")]
    secret: [u8; 32],
    httpd_url: String,
    external_url: String,
    database: DatabaseConfig,
}

fn secret_from_toml<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
    use serde::de::Error;
    let secret_base64 = String::deserialize(deserializer)?;
    let secret_decoded = base64::decode(&secret_base64).map_err(|err| Error::custom(err.to_string()))?;
    if secret_decoded.len() != 32 {
        Err(Error::custom("must decode to exactly 32 bytes"))
    } else {
        let mut secret = [0; 32];
        secret.copy_from_slice(&secret_decoded);
        Ok(secret)
    }
}

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        eprintln!("usage: ingestd-srt config");
        std::process::exit(1);
    }
    let config = {
        let config_filename = args.nth(1).unwrap();
        let config_toml = std::fs::read(config_filename).unwrap();
        toml::from_slice::<Config>(&config_toml).unwrap()
    };

    let listener = unsafe { UdpSocket::from_raw_fd(0) };
    let log_dir = Dir::open(config.stream_logs).unwrap();

    let mut bitmap_db_connection = block_on(SqliteConnection::connect(&config.database.url)).unwrap();
    let gpac_db_connection = block_on(SqliteConnection::connect(&config.database.url)).unwrap();
    let valid_stream_ids = Arc::new(ArcSwap::from_pointee(stream_db::generate_bitmap(&mut bitmap_db_connection)));

    let gpac_waker = Arc::new(AtomicWaker::new());
    let new_connections = Arc::new(Mutex::new(Vec::new()));

    srt::spawn_listen(listener, log_dir, config.secret, valid_stream_ids.clone(), gpac_waker.clone(), new_connections.clone()).unwrap();
    std::thread::Builder::new().name("sigusr1".to_string()).spawn(move || stream_db::listen_signal(valid_stream_ids, &mut bitmap_db_connection)).unwrap();
    smol::block_on(gpac::listen(gpac_waker, new_connections, gpac_db_connection, config.httpd_url, config.external_url));
}
