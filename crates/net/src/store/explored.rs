//! Reading and writing `explored.bin`: the explored ground of every body, in postcard, written
//! atomically beside the world file.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::ids::BodyId;
use crate::room::EXPLORED_WORDS;

/// Name of the explored ground file in the data folder.
pub const EXPLORED_FILE: &str = "explored.bin";

/// Returns the explored ground of `dir/explored.bin`, or nothing when there is no such file. A
/// bitset of the wrong size is left out.
pub(super) fn load(dir: &Path) -> io::Result<Vec<(BodyId, Vec<u64>)>> {
    let bytes = match fs::read(dir.join(EXPLORED_FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut explored: Vec<(BodyId, Vec<u64>)> = postcard::from_bytes(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    explored.retain(|(_, bits)| bits.len() == EXPLORED_WORDS);
    Ok(explored)
}

/// Writes `explored` to `dir/explored.bin` through a temporary file flushed to disk, so a
/// crash leaves either the old file or the new one.
pub(super) fn save(dir: &Path, explored: &[(BodyId, Vec<u64>)]) -> io::Result<()> {
    let bytes = postcard::to_stdvec(explored).map_err(io::Error::other)?;
    let tmp = dir.join(format!("{EXPLORED_FILE}.tmp"));
    let mut file = File::create(&tmp)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, dir.join(EXPLORED_FILE))?;
    File::open(dir)?.sync_all()
}
