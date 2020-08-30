use futures::task::AtomicWaker;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use crate::log::Logger;

pub struct NewConnection {
    pub logger: Logger,
    pub stream_id: Box<[u8]>,
    pub stream_uuid: Uuid,
    pub connection: Arc<Connection>,
}

#[derive(Default)]
pub struct Connection {
    pub gpac_waker: AtomicWaker,
    pub data: Mutex<Packets>,
}

#[derive(Default)]
pub struct Packets {
    pub packets: Vec<Packet>,
    pub closed: bool,
}

pub struct Packet {
    pub buffer: Vec<u8>,
    pub received: Instant,
}
