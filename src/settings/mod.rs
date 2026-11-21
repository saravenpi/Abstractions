//! User settings: loading, saving and applying them.
//!
//! Read settings with `Res<Persistent<Settings>>`. Change them with
//! `ResMut<Persistent<Settings>>` and call `persist()` to save.

mod apply;
mod audio;
mod gameplay;
mod lighting;
mod markers;
mod model;
mod prefs;
mod servers;
mod shadows;
mod ship_keys;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cloud_shade;
#[cfg(test)]
mod tests_lighting;
#[cfg(test)]
mod tests_markers;
#[cfg(test)]
mod tests_prefs;
#[cfg(test)]
mod tests_removed;
#[cfg(test)]
mod tests_servers;
#[cfg(test)]
mod tests_ship_keys;
#[cfg(test)]
mod tests_view;

pub use audio::{Audio, Chatter, Range};
pub use bevy_persistent::Persistent;
pub use gameplay::{Gameplay, InterfaceSize};
pub use lighting::{Lighting, Quality};
pub use markers::WorldMarkers;
pub use model::{Controls, Graphics, Keymap, Settings, WorldPrefs, render_step};
pub use servers::{SavedServer, default_servers};
pub use shadows::{ShadowDetail, ShadowReach};
pub use ship_keys::ShipKeys;

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
    app.add_plugins((apply::plugin, prefs::plugin));
}
