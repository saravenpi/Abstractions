//! The player body: walking, jumping, swimming, the flight gear, and what the camera follows.
//!
//! The camera itself belongs to [`crate::view`]: the controller reads the heading the rig
//! holds and tells the rig what to follow.

mod asks;
mod camera;
mod chase;
mod climb;
mod controller;
mod crouch;
mod dive;
mod eyes;
mod flight;
mod ground;
mod intent;
mod jet;
mod keys;
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

pub use asks::Intents;
pub use controller::Motion;
pub use sea::above_sea;
pub use seat::{place, seat};
pub(crate) use steer::wrap;
pub use steer::{Lead, Steer, Turn};
pub use wake::{awake, land};

use bevy::prelude::*;

use crate::sim::PlayerId;

/// Marker for a player body entity.
///
/// The body origin is the capsule center and its +Y is the local up. It carries the
/// [`PlayerId`] of its player, [`PlayerId::LOCAL`] unless it is given another, and the
/// [`Intents`] it moves by. The camera, marked [`crate::render::GameCamera`], is a child of the
/// local body that the camera rig places every frame.
#[derive(Component, Debug, Default, Clone, Copy)]
#[require(PlayerId, Intents)]
pub struct Player;

/// When the player systems run, in `Update`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    /// The keys and the view of this machine are sent as intents of the local player: after
    /// the mouse and the wing keys are read.
    Intend,
    /// The controller moves the body: after the mouse is read, before the camera rig runs.
    Move,
}

/// Radius of the player capsule in world units.
pub(crate) const BODY_RADIUS: f32 = 0.35;

/// Length of the cylindrical part of the player capsule in world units.
pub(crate) const BODY_LENGTH: f32 = 1.1;

/// Height of the eyes above the capsule center.
pub(crate) const EYE_OFFSET: f32 = 0.7;

/// Returns the item of the local player among `players`, paired with their ids. For the camera,
/// the interface and the input code, which follow the player at this machine.
pub fn local<T>(players: impl IntoIterator<Item = (PlayerId, T)>) -> Option<T> {
    players
        .into_iter()
        .find_map(|(id, item)| (id == PlayerId::LOCAL).then_some(item))
}

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
