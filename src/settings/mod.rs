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

/// Returns the game's directory inside the platform config directory.
pub fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("abstractions")
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

/// Loads the saved settings from the config directory.
///
/// Call it once and insert the result as a resource before the plugins that read it.
pub fn load() -> Persistent<Settings> {
    load_from(config_dir().join("settings.toml"))
}

/// Registers the systems that apply the settings. The resource itself comes from [`load`].
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(apply::plugin);
}
