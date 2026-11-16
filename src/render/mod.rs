//! Low-resolution game view, color banding, day cycle, sky, lighting, fog and camera effects.

mod camera;
mod celestial;
mod clock;
mod dome;
mod lighting;
mod lowres;
mod palette;
mod pixelate;
pub mod ray_tracing;
mod sky;

pub use camera::{configure_camera, set_fov};
pub use celestial::{daylight, moon_direction, sun_direction};
pub use clock::DayClock;
pub use lighting::{Moon, Sun};
pub use lowres::LowRes;
pub use palette::Palette;
pub use pixelate::Pixelate;
pub use ray_tracing::RayTracing;

use bevy::prelude::*;

/// Marker for the player's first-person 3D camera.
///
/// The render module adds post-processing, fog, field of view and effects to any entity with it.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GameCamera;

/// Registers the render plugins.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        pixelate::plugin,
        lowres::plugin,
        clock::plugin,
        palette::plugin,
        lighting::plugin,
        sky::plugin,
        dome::plugin,
        camera::plugin,
        ray_tracing::plugin,
    ));
}
