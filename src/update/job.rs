//! One install from start to end: download, check, then put in place. It runs on its own thread.

use std::fs;
use std::path::Path;

use super::allow;
use super::checksum;
use super::download::download;
use super::failure::Failure;
use super::host::Host;
use super::install::swap;
use super::place::{Place, part_path, writable};
use super::platform::{Platform, pick};
use super::release::{Asset, Release};
use super::signals::Signals;
use super::state::{Outcome, Plan};

/// Largest checksum list accepted, in bytes.
const SUMS_LIMIT: u64 = 64 * 1024;

/// Why the game does not install by itself when releases have no build for this computer.
pub const NO_BUILD: &str = "Releases have no build for this computer.";
/// Why the game does not install by itself when a release lacks the plain file or its checksums.
pub const NO_FILE: &str = "This release has no file the game can install by itself.";
/// Why the game does not install by itself when its folder is read-only.
pub const READ_ONLY: &str = "The game cannot write to its own folder: it was installed by a \
package manager, or the folder is read-only.";
/// Why the game does not download the disk image when there is no Downloads folder to write to.
pub const NO_DOWNLOADS: &str = "The game cannot write to your Downloads folder.";

/// Returns how the game at `place` can install `release`.
pub fn plan(place: &Place, release: &Release) -> Plan {
    let Some(platform) = place.platform else {
        return Plan::Page(NO_BUILD);
    };
    if pick(release, platform).is_none() {
        return Plan::Page(NO_FILE);
    }
    let folder = place.exe.parent().is_some_and(writable);
    let downloads = place.downloads.as_deref().is_some_and(writable);
    match (platform, folder, downloads) {
        (Platform::Mac, _, true) => Plan::Image,
        (Platform::Mac, _, false) => Plan::Page(NO_DOWNLOADS),
        (_, true, _) => Plan::Swap,
        (_, false, _) => Plan::Page(READ_ONLY),
    }
}

/// Downloads the first of `files` to `part` and checks it against the second, its checksum list.
fn fetch(
    host: &dyn Host,
    files: (&Asset, &Asset),
    part: &Path,
    signals: &Signals,
) -> Result<(), Failure> {
    let (payload, sums) = files;
    download(host, payload, part, signals)?;
    if !allow::download(&sums.url) {
        return Err(Failure::Forbidden);
    }
    let list = host.get(&sums.url, SUMS_LIMIT)?;
    checksum::verify(part, &String::from_utf8_lossy(&list), &payload.name)
}

/// Puts the checked download in place, unless the player cancelled first. From here on the
/// install cannot be cancelled.
fn settle(
    platform: Platform,
    part: &Path,
    target: &Path,
    signals: &Signals,
) -> Result<Outcome, Failure> {
    if !signals.commit() {
        return Err(Failure::Cancelled);
    }
    if platform == Platform::Mac {
        fs::rename(part, target)?;
        return Ok(Outcome::Image(target.to_path_buf()));
    }
    swap(platform, part, target)?;
    Ok(Outcome::Swapped)
}

/// Installs `release` for the game at `place`: downloads the file of its platform next to where
/// it goes, checks its size and checksum, then swaps the executable (Linux, Windows) or leaves
/// the disk image in Downloads (macOS). Any failure removes the download and changes nothing else.
pub fn install(
    host: &dyn Host,
    place: &Place,
    release: &Release,
    signals: &Signals,
) -> Result<Outcome, Failure> {
    let platform = place.platform.ok_or(Failure::Unsupported)?;
    let (payload, sums) = pick(release, platform).ok_or(Failure::Unsupported)?;
    let target = match (platform, &place.downloads) {
        (Platform::Mac, Some(folder)) => folder.join(&payload.name),
        (Platform::Mac, None) => return Err(Failure::Unsupported),
        _ => place.exe.clone(),
    };
    let part = part_path(&target);
    let done = fetch(host, (&payload, &sums), &part, signals)
        .and_then(|()| settle(platform, &part, &target, signals));
    if done.is_err() {
        let _ = fs::remove_file(&part);
    }
    done
}
