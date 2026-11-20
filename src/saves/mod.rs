//! Saved worlds: files on disk, the active world, and autosave.

mod autosave;
mod capture;
mod cli;
#[cfg(test)]
mod cli_tests;
mod format;
mod id;
#[cfg(test)]
mod log_tests;
mod model;
#[cfg(test)]
mod moon_tests;
#[cfg(test)]
mod old_tests;
mod resume;
mod snapshot;
mod start;
mod store;
#[cfg(test)]
mod store_tests;
#[cfg(test)]
mod tests_far;
mod writer;

pub use autosave::{SaveSet, SavedFlash};
pub use format::{ago, play_time};
pub use model::Explored as SavedGround;
pub use model::Pin;
pub use model::{GAME_VERSION, OLDER_VERSION, Place, SAVE_VERSION, WorldEntry, WorldSave};
pub use snapshot::Snapshot;
pub use start::{Starter, unplayed};
pub use store::{delete, list, new_entry, now, write};
pub use writer::Writer;

use std::path::PathBuf;

use bevy::prelude::*;

use crate::screens::Screen;

/// The world being played. Exists from the moment a world starts until the title screen returns.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ActiveWorld(pub WorldEntry);

/// Marks an active world that lives on a server: it is played as any other, but never written
/// to the local saves folder.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Unsaved;

/// Directory that holds the world files.
#[derive(Resource, Debug, Clone)]
pub struct SaveDir(pub PathBuf);

impl Default for SaveDir {
    fn default() -> Self {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        Self(base.join("abstractions").join("worlds"))
    }
}

/// Forgets the active world once the title screen is back.
fn clear_active(mut commands: Commands) {
    commands.remove_resource::<ActiveWorld>();
    commands.remove_resource::<Unsaved>();
}

/// Registers the save directory, autosave and active world cleanup.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SaveDir>()
        .add_plugins((autosave::plugin, writer::plugin, cli::plugin))
        .add_systems(OnEnter(Screen::Title), clear_active);
}
