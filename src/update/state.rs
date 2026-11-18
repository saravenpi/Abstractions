//! What the updater knows: whether a newer release exists, and how far an install has gone.

use std::path::PathBuf;

use super::release::Release;

/// The result of looking for a newer release.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    /// Nothing was checked yet.
    #[default]
    Idle,
    /// A check is running.
    Checking,
    /// The game is the latest release.
    UpToDate,
    /// A newer release exists.
    Available(Release),
    /// The check failed, for this reason.
    Failed(String),
}

/// How the game can install a release on this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Plan {
    /// Replace the game's executable in its folder.
    #[default]
    Swap,
    /// Download the disk image and let the player install it: macOS.
    Image,
    /// The game cannot install it by itself, for this reason: offer the download page.
    Page(&'static str),
}

/// What a finished install leaves to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The new game is in place: restart to run it.
    Swapped,
    /// The disk image is at this path: open it and drag the application.
    Image(PathBuf),
}

/// How far an install has gone.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Stage {
    /// No install was started, or the last one was cancelled.
    #[default]
    Idle,
    /// The file is being downloaded and checked.
    Downloading,
    /// The install is done.
    Ready(Outcome),
    /// The install failed, for this reason. The installed game is as it was.
    Failed(String),
}
