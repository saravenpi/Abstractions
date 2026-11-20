//! Reading and writing `hunt3.bin`: the carcasses and the stuck spears of a server, in
//! postcard, written atomically beside the world file.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::wire::HuntWorld;

/// Name of the hunt file in the data folder.
pub const HUNT_FILE: &str = "hunt3.bin";

/// Returns the hunt of `dir/hunt3.bin`, or an empty one when there is no such file.
pub(super) fn load(dir: &Path) -> io::Result<HuntWorld> {
    let bytes = match fs::read(dir.join(HUNT_FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(HuntWorld::default()),
        Err(error) => return Err(error),
    };
    postcard::from_bytes(&bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

/// Writes `hunt` to `dir/hunt3.bin` through a temporary file flushed to disk, so a crash leaves
/// either the old file or the new one.
pub(super) fn save(dir: &Path, hunt: &HuntWorld) -> io::Result<()> {
    let bytes = postcard::to_stdvec(hunt).map_err(io::Error::other)?;
    let tmp = dir.join(format!("{HUNT_FILE}.tmp"));
    let mut file = File::create(&tmp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, dir.join(HUNT_FILE))?;
    File::open(dir)?.sync_all()
}
