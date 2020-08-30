use arc_swap::ArcSwap;
use futures::executor::block_on_stream;
use roaring::RoaringBitmap;
use signal_hook::iterator::Signals;
use signal_hook::SIGUSR1;
use std::sync::Arc;
use sqlx::{SqliteConnection, query};

pub fn listen_signal(valid_stream_ids: Arc<ArcSwap<RoaringBitmap>>, mut db: &mut SqliteConnection) {
    let signals = Signals::new(&[SIGUSR1]).unwrap();
    for _ in signals.forever() {
        valid_stream_ids.swap(Arc::new(generate_bitmap(&mut db)));
    }
}

pub fn generate_bitmap(db: &mut SqliteConnection) -> RoaringBitmap {
    let stream_ids = query!("SELECT id from streams WHERE active = TRUE").fetch(db);
    let mut bitmap = RoaringBitmap::new();
    for res in block_on_stream(stream_ids) {
        match res {
            Ok(id) => {
                bitmap.insert(id.id as u32);
            },
            Err(e) => {
                panic!(e);
            }
        }
    }
    bitmap
}
