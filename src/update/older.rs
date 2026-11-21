//! "Open with 0.10": running a world of an older release with that release's own build.
//!
//! The game downloads the build of the release for this platform into a folder of the game's
//! data directory, checks its SHA-256 against the release's checksum list, and starts it with
//! `--world <id>`. Releases that do not parse `--world` (such as `v0.10.1`) open to their world
//! list instead. A build already there and still matching its checksum is not downloaded again.

use std::fs;
use std::path::{Path, PathBuf};

use super::allow;
use super::checksum;
use super::download::download;
use super::failure::Failure;
use super::host::Host;
use super::install::mark_executable;
use super::place::{Place, part_path};
use super::platform::{Platform, pick};
use super::release::{self, Asset};
use super::run::background;
use super::signals::Signals;
use super::updates::{Event, Updates};

/// The release that opens worlds made by 0.10. It ignores `--world` and opens to its world list.
pub const OLDER_TAG: &str = "v0.10.1";

/// Where GitHub describes one release of the game, by tag.
pub const TAGS: &str = "https://api.github.com/repos/saravenpi/Abstractions/releases/tags/";

/// Largest release document or checksum list accepted, in bytes.
const LIMIT: u64 = 1024 * 1024;

/// How far opening a world with an older release has gone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Older {
    /// Nothing was asked.
    #[default]
    Idle,
    /// The build is being fetched and checked.
    Working,
    /// The older game was started.
    Opened,
    /// It failed, for this reason.
    Failed(String),
}

/// Returns the folder that holds the build of the release tagged `tag`, inside `data`.
pub fn folder(data: &Path, tag: &str) -> PathBuf {
    data.join("older").join(tag)
}

/// Returns the build's file and its checksum list, as the release tagged `tag` has them for
/// the platform.
fn files(host: &dyn Host, platform: Platform, tag: &str) -> Result<(Asset, Asset), Failure> {
    let json = host.get(&format!("{TAGS}{tag}"), LIMIT)?;
    pick(&release::parse(&json)?, platform).ok_or(Failure::Unsupported)
}

/// Puts the build `payload` at `dest`, checked against `sums`: a copy already there that
/// matches is kept, and anything else is downloaded to a part file and moved in once it
/// matches.
fn fetch(
    host: &dyn Host,
    (payload, sums): (&Asset, &Asset),
    dest: &Path,
    signals: &Signals,
) -> Result<(), Failure> {
    if !allow::download(&sums.url) {
        return Err(Failure::Forbidden);
    }
    let list = String::from_utf8_lossy(&host.get(&sums.url, LIMIT)?).into_owned();
    if checksum::verify(dest, &list, &payload.name).is_ok() {
        return Ok(());
    }
    let part = part_path(dest);
    download(host, payload, &part, signals)?;
    let moved = checksum::verify(&part, &list, &payload.name)
        .and_then(|()| fs::rename(&part, dest).map_err(Failure::from));
    if moved.is_err() {
        let _ = fs::remove_file(&part);
    }
    moved
}

/// Opens the world with id `world` in the build of the release tagged `tag`, kept in `data`,
/// passing `--world <world>` (releases such as `v0.10.1` open to their world list). Fails without
/// starting anything when the release has no build for this computer, a download is cut or
/// damaged, or the player cancels.
pub fn open(
    host: &dyn Host,
    place: &Place,
    data: &Path,
    (tag, world): (&str, &str),
    signals: &Signals,
) -> Result<(), Failure> {
    let platform = place.platform.ok_or(Failure::Unsupported)?;
    if platform == Platform::Mac || world.is_empty() {
        return Err(Failure::Unsupported);
    }
    let (payload, sums) = files(host, platform, tag)?;
    let dest = folder(data, tag).join(&payload.name);
    fs::create_dir_all(folder(data, tag))?;
    fetch(host, (&payload, &sums), &dest, signals)?;
    mark_executable(&dest, &place.exe)?;
    host.launch(&dest, &["--world".to_string(), world.to_string()])
}

impl Updates {
    /// Returns how far opening a world with an older release has gone.
    pub fn older(&self) -> &Older {
        &self.older
    }

    /// Opens the world `world` with the older release tagged `tag`, in the background: fetches
    /// its build into `data` when it is not there, then starts it. Does nothing while one runs.
    pub fn open_older(&mut self, tag: &str, world: &str, data: &Path) {
        if self.older == Older::Working {
            return;
        }
        let (host, place, sender) = (self.host.clone(), self.place.clone(), self.sender.clone());
        let (tag, world, data) = (tag.to_string(), world.to_string(), data.to_path_buf());
        let started = background("update-older", move || {
            let signals = Signals::default();
            let done = open(&*host, &place, &data, (&tag, &world), &signals);
            let _ = sender.send(Event::Opened(done));
        });
        self.older = match started {
            true => Older::Working,
            false => Older::Failed(Failure::Disk("no thread".into()).reason()),
        };
    }

    /// Takes in the end of opening a world with an older release.
    pub(super) fn opened(&mut self, done: Result<(), Failure>) {
        self.older = match done {
            Ok(()) => Older::Opened,
            Err(failure) => Older::Failed(failure.reason()),
        };
    }
}
