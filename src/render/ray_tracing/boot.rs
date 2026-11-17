//! The boot guard: a marker file that exists while a start with the Solari plugins is unproven.

use std::path::Path;

use bevy::app::AppExit;
use bevy::prelude::*;

use super::RayTracing;
use crate::render::GameCamera;
use crate::settings::config_dir;

/// Name of the marker file, kept next to the settings file.
pub const MARKER: &str = "ray-tracing-starting";

/// Frames that count a start as proven.
const PROVEN_FRAMES: u32 = 120;

/// What the boot guard decided for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// The Solari plugins start, and the marker file now exists.
    Armed,
    /// The last start with the plugins never finished, so this run skips them.
    Recovered,
    /// The marker file could not be written, so this run skips the plugins.
    Unwritable,
    /// The environment switch keeps the plugins out of this run.
    Disabled,
}

/// Present while a start is unproven; holds the frames counted toward the proof.
#[derive(Resource, Debug, Clone, Default)]
pub struct BootMarker(pub u32);

/// Writes the marker file in `dir`.
fn arm(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir).and_then(|()| std::fs::write(dir.join(MARKER), b""))
}

/// Decides this run's start from the marker file in `dir` and leaves a new marker when the plugins may start.
pub fn begin(dir: &Path) -> Start {
    let marker = dir.join(MARKER);
    if marker.exists() {
        if let Err(error) = std::fs::remove_file(&marker) {
            warn!("could not remove {}: {error}", marker.display());
        }
        warn!("the last start with the Solari plugins did not finish, so they are skipped");
        return Start::Recovered;
    }
    match arm(dir) {
        Ok(()) => Start::Armed,
        Err(error) => {
            warn!("could not write the boot marker, so the Solari plugins are skipped: {error}");
            Start::Unwritable
        }
    }
}

/// Starts a new proof when ray tracing is switched on while the game runs.
pub fn rearm(commands: &mut Commands, dir: &Path) {
    match arm(dir) {
        Ok(()) => commands.insert_resource(BootMarker::default()),
        Err(error) => warn!("could not write the boot marker: {error}"),
    }
}

/// Removes the marker file and forgets it.
fn clear(commands: &mut Commands) {
    let path = config_dir().join(MARKER);
    if let Err(error) = std::fs::remove_file(&path) {
        warn!("could not remove {}: {error}", path.display());
    }
    commands.remove_resource::<BootMarker>();
}

/// Counts frames and clears the marker once enough passed: with ray tracing on they must show a game camera.
fn prove(
    mut commands: Commands,
    mut marker: ResMut<BootMarker>,
    cameras: Query<(), With<GameCamera>>,
    ray_tracing: Res<RayTracing>,
) {
    if !ray_tracing.0 || !cameras.is_empty() {
        marker.0 += 1;
    }
    if marker.0 >= PROVEN_FRAMES {
        clear(&mut commands);
    }
}

/// Clears the marker when the game exits cleanly.
///
/// An exit with an error keeps it: Bevy quits that way on a render error, and the next start then skips the
/// plugins and turns the setting off.
fn clear_on_exit(mut commands: Commands, mut exits: MessageReader<AppExit>) {
    if exits.read().next().is_some_and(AppExit::is_success) {
        clear(&mut commands);
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

    /// A start leaves a marker, a second start finds it and recovers, a third starts again.
    #[test]
    fn a_dead_start_is_recovered_once() {
        let dir = std::env::temp_dir().join(format!("abstractions-boot-{}", std::process::id()));
        assert_eq!(begin(&dir), Start::Armed);
        assert!(dir.join(MARKER).exists());
        assert_eq!(begin(&dir), Start::Recovered);
        assert!(!dir.join(MARKER).exists());
        assert_eq!(begin(&dir), Start::Armed);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
