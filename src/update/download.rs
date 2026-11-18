//! Downloading one release file to disk, in chunks, with progress, a size cap and cancel.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::allow;
use super::failure::Failure;
use super::host::Host;
use super::release::Asset;
use super::signals::Signals;

/// Largest file the updater accepts: 500 MB.
pub const CAP: u64 = 500 * 1024 * 1024;

/// Streams `asset` into `file`, counting bytes. Stops when cancelled, or when more arrives than
/// the release declares. Returns the number of bytes written.
fn receive(
    host: &dyn Host,
    asset: &Asset,
    file: &mut File,
    signals: &Signals,
) -> Result<u64, Failure> {
    let mut received = 0u64;
    host.stream(&asset.url, &mut |chunk| {
        if signals.cancelled() {
            return Err(Failure::Cancelled);
        }
        received += chunk.len() as u64;
        if received > asset.size {
            return Err(Failure::Size);
        }
        file.write_all(chunk)?;
        signals.progress(received);
        Ok(())
    })?;
    file.flush()?;
    Ok(received)
}

/// Downloads `asset` to `dest`. The address must be on the game's release page and the size
/// under [`CAP`]; the file must end with exactly the declared size. On any failure, and on
/// cancel, `dest` is removed.
pub fn download(
    host: &dyn Host,
    asset: &Asset,
    dest: &Path,
    signals: &Signals,
) -> Result<(), Failure> {
    if !allow::download(&asset.url) {
        return Err(Failure::Forbidden);
    }
    if asset.size == 0 || asset.size > CAP {
        return Err(Failure::TooLarge);
    }
    let mut file = File::create(dest)?;
    let received = receive(host, asset, &mut file, signals);
    drop(file);
    let whole = received.and_then(|bytes| match bytes == asset.size {
        true => Ok(()),
        false => Err(Failure::Size),
    });
    if whole.is_err() {
        let _ = fs::remove_file(dest);
    }
    whole
}
