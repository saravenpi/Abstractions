//! Pixelate post-process, lighting and fog.

mod lighting;
mod pixelate;

pub use lighting::{SKY_COLOR, Sun, fog_falloff};
pub use pixelate::Pixelate;

use bevy::prelude::*;

/// Marker for the player's first-person 3D camera.
///
/// The render module adds post-processing, fog and tonemapping to any entity with it.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GameCamera;

/// Registers the render plugins.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((pixelate::plugin, lighting::plugin));
}
