//! First-person player: movement, jumping, swimming and mouse look.

mod camera;
mod controller;
mod dive;
mod ground;
mod intent;
mod seat;
mod spawn;
mod swim;
mod upright;

#[cfg(test)]
mod tests;

pub use controller::Motion;
pub use seat::{place, seat};

use bevy::prelude::*;

/// Marker for the player body entity.
///
/// The body origin is the capsule center and its +Y is the local up. The camera, marked
/// [`crate::render::GameCamera`], is a child at [`EYE_OFFSET`] above it.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Player;

/// Radius of the player capsule in world units.
pub(crate) const BODY_RADIUS: f32 = 0.35;

/// Length of the cylindrical part of the player capsule in world units.
pub(crate) const BODY_LENGTH: f32 = 1.1;

/// Height of the eyes above the capsule center.
pub(crate) const EYE_OFFSET: f32 = 0.7;

/// Registers the player systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((spawn::plugin, controller::plugin, camera::plugin));
}
