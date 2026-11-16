//! The boot guard: a marker file that exists while a ray traced start is unproven.

use std::path::{Path, PathBuf};

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::render::GameCamera;

/// Name of the marker file, kept next to the settings file.
pub const MARKER: &str = "ray-tracing-starting";

/// Frames rendered with a game camera before a ray traced start counts as proven.
const PROVEN_FRAMES: u32 = 120;

/// What the boot guard decided for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// Ray tracing was not requested.
    Off,
    /// Ray tracing starts, and the marker file now exists.
    On,
    /// The previous ray traced start never finished, so this run skips it.
    Recovered,
}

/// Where the marker file lives while a ray traced start is unproven.
#[derive(Resource, Debug, Clone)]
pub struct BootMarker(pub PathBuf);

/// Returns whether ray tracing may start: it is requested and no earlier attempt died.
pub fn allowed(setting: bool, marker_exists: bool) -> bool {
    setting && !marker_exists
}

/// Decides this run's start and leaves the marker file in `dir` when ray tracing starts.
pub fn begin(setting: bool, dir: &Path) -> Start {
    let marker = dir.join(MARKER);
    let exists = marker.exists();
    if exists && let Err(error) = std::fs::remove_file(&marker) {
        warn!("could not remove {}: {error}", marker.display());
    }
    if !setting {
        return Start::Off;
    }
    if !allowed(setting, exists) {
        warn!("the last ray traced start did not finish, so ray tracing is turned off");
        return Start::Recovered;
    }
    let written = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&marker, b""));
    match written {
        Ok(()) => Start::On,
        Err(error) => {
            warn!("could not write the boot marker, so ray tracing is turned off: {error}");
            Start::Off
        }
    }
}

/// Removes the marker file and forgets it.
fn clear(commands: &mut Commands, marker: &BootMarker) {
    if let Err(error) = std::fs::remove_file(&marker.0) {
        warn!("could not remove {}: {error}", marker.0.display());
    }
    commands.remove_resource::<BootMarker>();
}

/// Clears the marker once a game camera has rendered enough frames.
fn prove(
    mut commands: Commands,
    marker: Res<BootMarker>,
    cameras: Query<(), With<GameCamera>>,
    mut frames: Local<u32>,
) {
    if !cameras.is_empty() {
        *frames += 1;
    }
    if *frames >= PROVEN_FRAMES {
        clear(&mut commands, &marker);
    }
}

/// Clears the marker when the game exits cleanly.
fn clear_on_exit(
    mut commands: Commands,
    marker: Res<BootMarker>,
    mut exits: MessageReader<AppExit>,
) {
    if exits.read().next().is_some() {
        clear(&mut commands, &marker);
    }
}

/// Registers the systems that clear the marker.
pub(super) fn plugin(app: &mut App) {
    let marked = resource_exists::<BootMarker>;
    app.add_systems(Update, prove.run_if(marked))
        .add_systems(Last, clear_on_exit.run_if(marked));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only a requested start without a leftover marker is allowed.
    #[test]
    fn allowed_needs_a_request_and_no_marker() {
        assert!(allowed(true, false));
        assert!(!allowed(true, true));
        assert!(!allowed(false, false));
        assert!(!allowed(false, true));
    }

    /// A start leaves a marker, a second start finds it and recovers, a third starts again.
    #[test]
    fn a_dead_start_is_recovered_once() {
        let dir = std::env::temp_dir().join(format!("abstractions-boot-{}", std::process::id()));
        assert_eq!(begin(false, &dir), Start::Off);
        assert_eq!(begin(true, &dir), Start::On);
        assert!(dir.join(MARKER).exists());
        assert_eq!(begin(true, &dir), Start::Recovered);
        assert!(!dir.join(MARKER).exists());
        assert_eq!(begin(true, &dir), Start::On);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
