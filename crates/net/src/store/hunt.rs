//! Reading and writing the binary files beside the world file: `hunt3.bin` holds the carcasses and
//! the stuck spears of a server, `ground3.bin` the things on its ground, each in postcard, written
//! atomically.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Name of the hunt file in the data folder.
pub const HUNT_FILE: &str = "hunt3.bin";

/// Name of the ground file in the data folder.
pub const GROUND_FILE: &str = "ground3.bin";

/// Returns what `dir/name` holds, or the default when there is no such file.
pub(super) fn load<T: DeserializeOwned + Default>(dir: &Path, name: &str) -> io::Result<T> {
    let bytes = match fs::read(dir.join(name)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(T::default()),
        Err(error) => return Err(error),
    };
    postcard::from_bytes(&bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Writes `value` to `dir/name` through a temporary file flushed to disk, so a crash leaves either
/// the old file or the new one.
pub(super) fn save<T: Serialize>(dir: &Path, name: &str, value: &T) -> io::Result<()> {
    let bytes = postcard::to_stdvec(value).map_err(io::Error::other)?;
    let tmp = dir.join(format!("{name}.tmp"));
    let mut file = File::create(&tmp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, dir.join(name))?;
    File::open(dir)?.sync_all()
}
