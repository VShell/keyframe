use anyhow::{Result, bail};
use std::path::PathBuf;

mod driver;
mod read;
mod ring;

pub use driver::{Driver, spawn_driver};
pub use read::{alloc_read_buffer, read_from_file};
pub use ring::write_from_ring;

use crate::State;

pub fn get_file_path(state: &State, path: &str) -> Result<PathBuf> {
    let decoded_path = percent_encoding::percent_decode_str(path).decode_utf8()?;
    let stripped_path = decoded_path.trim_start_matches('/');
    let cleaned_path = path_clean::clean(&stripped_path);
    if cleaned_path.starts_with("../") {
        bail!("invalid path");
    }
    Ok(state.file_root.join(cleaned_path))
}
