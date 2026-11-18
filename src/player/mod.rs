//! The player body: walking, jumping, swimming, the flight gear, and what the camera follows.
//!
//! The camera itself belongs to [`crate::view`]: the controller reads the heading the rig
//! holds and tells the rig what to follow.

mod camera;
mod controller;
mod dive;
mod eyes;
mod flight;
mod ground;
mod intent;
mod lead;
mod sea;
mod seat;
mod spawn;
mod steer;
mod swim;
mod upright;
mod wake;
mod walk;

#[cfg(test)]
mod tests;

pub use controller::Motion;
pub use sea::above_sea;
pub use seat::{place, seat};
pub(crate) use steer::wrap;
pub use steer::{Lead, Steer, Turn};
pub use wake::awake;

use bevy::prelude::*;

/// Marker for the player body entity.
///
/// The body origin is the capsule center and its +Y is the local up. The camera, marked
/// [`crate::render::GameCamera`], is a child of it that the camera rig places every frame.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Player;

/// When the player systems run, in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    /// The controller moves the body: after the mouse is read, before the camera rig runs.
    Move,
}

/// Radius of the player capsule in world units.
pub(crate) const BODY_RADIUS: f32 = 0.35;

/// Length of the cylindrical part of the player capsule in world units.
pub(crate) const BODY_LENGTH: f32 = 1.1;

/// Height of the eyes above the capsule center.
pub(crate) const EYE_OFFSET: f32 = 0.7;

/// Registers the player systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<crate::view::Heading>()
        .init_resource::<crate::view::Look>()
        .init_resource::<crate::view::ViewPrefs>()
        .add_message::<crate::view::Cut>()
        .add_plugins((
            spawn::plugin,
            controller::plugin,
            camera::plugin,
            eyes::plugin,
            wake::plugin,
        ));
}
