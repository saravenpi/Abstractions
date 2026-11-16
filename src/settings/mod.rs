//! User settings: loading, saving and applying them.
//!
//! Read settings with `Res<Persistent<Settings>>`. Change them with
//! `ResMut<Persistent<Settings>>` and call `persist()` to save.

mod apply;
mod model;
#[cfg(test)]
mod tests;

pub use bevy_persistent::Persistent;
pub use model::{Audio, Controls, Graphics, Keymap, Settings, WorldPrefs};

use bevy::prelude::*;
use bevy_persistent::StorageFormat;

/// Returns the settings file path inside the platform config directory.
fn settings_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("abstractions")
        .join("settings.toml")
}

/// Loads settings from a file, falling back to defaults when missing or broken.
fn load_from(path: std::path::PathBuf) -> Persistent<Settings> {
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(path)
        .default(Settings::default())
        .revertible(true)
        .revert_to_default_on_deserialization_errors(true)
        .build()
        .expect("settings storage must be writable")
}

/// Inserts the persisted settings resource.
pub(super) fn plugin(app: &mut App) {
    app.insert_resource(load_from(settings_path()));
    app.add_plugins(apply::plugin);
}
