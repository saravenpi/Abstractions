//! Reading and writing `explored3.bin`: the explored ground of every body, in postcard, written
//! atomically beside the world file.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::ids::BodyId;
use crate::leaves::Run;

/// Name of the explored ground file in the data folder.
pub const EXPLORED_FILE: &str = "explored3.bin";

/// Returns the explored ground of `dir/explored3.bin`, or nothing when there is no such file.
pub(super) fn load(dir: &Path) -> io::Result<Vec<(BodyId, Vec<Run>)>> {
    let bytes = match fs::read(dir.join(EXPLORED_FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let explored: Vec<(BodyId, Vec<Run>)> = postcard::from_bytes(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(explored)
}

/// Writes `explored` to `dir/explored3.bin` through a temporary file flushed to disk, so a
/// crash leaves either the old file or the new one.
pub(super) fn save(dir: &Path, explored: &[(BodyId, Vec<Run>)]) -> io::Result<()> {
    let bytes = postcard::to_stdvec(explored).map_err(io::Error::other)?;
    let tmp = dir.join(format!("{EXPLORED_FILE}.tmp"));
    let mut file = File::create(&tmp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, dir.join(EXPLORED_FILE))?;
    File::open(dir)?.sync_all()
}
