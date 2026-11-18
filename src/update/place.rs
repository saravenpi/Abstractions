//! Where the running game lives on this computer, and the file names an update uses beside it.

use std::fs;
use std::path::{Path, PathBuf};

use super::platform::Platform;

/// Where the game runs: what the updater needs to know to replace it and start it again.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    /// The platform releases are built for, or nothing when they have no build for this one.
    pub platform: Option<Platform>,
    /// The game's executable, read once at start: after an update the system reports another.
    pub exe: PathBuf,
    /// The user's Downloads folder, where a disk image goes.
    pub downloads: Option<PathBuf>,
    /// The arguments the game was started with, passed on at a restart.
    pub args: Vec<String>,
}

impl Place {
    /// Returns where this game runs.
    pub fn current() -> Self {
        Self {
            platform: Platform::current(),
            exe: std::env::current_exe().unwrap_or_default(),
            downloads: dirs::download_dir(),
            args: std::env::args().skip(1).collect(),
        }
    }
}

/// Returns `path` with `tag` put before its extension: `abstractions.old`, `abstractions.old.exe`.
fn tagged(path: &Path, tag: &str) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let name = match path.extension() {
        Some(extension) => format!("{stem}.{tag}.{}", extension.to_string_lossy()),
        None => format!("{stem}.{tag}"),
    };
    path.with_file_name(name)
}

/// Returns where the previous game is kept after an update of the game at `exe`.
pub fn old_path(exe: &Path) -> PathBuf {
    tagged(exe, "old")
}

/// Returns where a download meant for `target` is written until it is checked: next to it, so
/// that moving it into place stays on one file system.
pub fn part_path(target: &Path) -> PathBuf {
    tagged(target, "part")
}

/// Returns whether files can be created in `dir`, by creating one and removing it.
pub fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".abstractions-probe-{}", std::process::id()));
    let made = fs::File::create(&probe).is_ok();
    let _ = fs::remove_file(&probe);
    made
}
