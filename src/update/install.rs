//! Putting a downloaded game in place of the running one. Every function takes paths, so the
//! strategies run on any folder.
//!
//! The previous game is kept next to the new one as `abstractions.old` (`abstractions.old.exe`
//! on Windows) until the new one has started once.

use std::fs;
use std::io;
use std::path::Path;

use super::place::{old_path, part_path};
use super::platform::Platform;

/// Lets `file` be run: it takes the permissions of the game at `like`, with the run bits set.
/// Windows has no such bits, so nothing happens there.
#[cfg(unix)]
pub fn mark_executable(file: &Path, like: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(like).map_or(0o755, |meta| meta.permissions().mode());
    fs::set_permissions(file, fs::Permissions::from_mode(mode | 0o111))
}

/// Lets `file` be run. Windows has no run bits, so nothing happens.
#[cfg(not(unix))]
pub fn mark_executable(_: &Path, _: &Path) -> io::Result<()> {
    Ok(())
}

/// Replaces a running game the Linux way: the old file gets a second name, then the new file is
/// renamed over the first name in one step. At no time is there no game at `exe`.
pub fn swap_over(new: &Path, exe: &Path) -> io::Result<()> {
    let old = old_path(exe);
    let _ = fs::remove_file(&old);
    if fs::hard_link(exe, &old).is_err() {
        fs::copy(exe, &old)?;
    }
    fs::rename(new, exe).inspect_err(|_| {
        let _ = fs::remove_file(&old);
    })
}

/// Replaces a running game the Windows way, where a running file cannot be overwritten but can
/// be renamed: the old file moves aside, then the new one takes its name. If the second step
/// fails the old file moves back.
pub fn swap_aside(new: &Path, exe: &Path) -> io::Result<()> {
    let old = old_path(exe);
    let _ = fs::remove_file(&old);
    fs::rename(exe, &old)?;
    fs::rename(new, exe).inspect_err(|_| {
        let _ = fs::rename(&old, exe);
    })
}

/// Puts the checked download `new` in place of the game at `exe`, the way `platform` allows.
pub fn swap(platform: Platform, new: &Path, exe: &Path) -> io::Result<()> {
    mark_executable(new, exe)?;
    match platform {
        Platform::Windows => swap_aside(new, exe),
        Platform::Linux | Platform::Mac => swap_over(new, exe),
    }
}

/// Removes what an earlier update left next to the game at `exe`: the previous game and any
/// unfinished download. Called once the game has started, which shows the new one works.
pub fn sweep(exe: &Path) {
    if exe.file_name().is_some() {
        let _ = fs::remove_file(old_path(exe));
        let _ = fs::remove_file(part_path(exe));
    }
}
