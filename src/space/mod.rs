//! Where things are: universe positions, coordinate frames and the render anchor.

mod anchor;
mod clock;
#[cfg(test)]
mod clock_tests;
mod frame;
mod gravity;
mod local;
mod pos;
mod rebase;

#[cfg(test)]
mod tests;

pub use anchor::{Anchor, Frame};
pub use clock::Clock;
pub use frame::{FrameSet, on_foot};
pub use gravity::local_gravity;
pub use local::{altitude, up};
pub use pos::UniPos;
pub use rebase::{Shift, rebase_limit, rebased};

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use crate::universe::BodyId;

/// Pose of an entity in the frame of the [`Anchor`], copied into its `Transform` every frame.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Position in metres in the anchor frame.
    pub pos: DVec3,
    /// Orientation in the anchor frame.
    pub rot: DQuat,
}

/// Marker for a free-moving entity whose `Transform` is the truth.
///
/// When the anchor changes, the entity is moved by the difference so that it stays where it was.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Rebased;

/// How the player travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Travel {
    /// Walking or swimming.
    #[default]
    OnFoot,
    /// Seated in the ship.
    InShip,
}

/// Where the player is.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    /// Planet the player stands on, or the nearest one while in space.
    pub body: BodyId,
    /// Whether the player is on foot or in the ship.
    pub travel: Travel,
}

/// Registers the space systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((clock::plugin, frame::plugin, gravity::plugin));
}
