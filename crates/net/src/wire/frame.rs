//! The frames a position on the wire is expressed in, and the kinds of things that have one.

use serde::{Deserialize, Serialize};

use crate::ids::{BodyId, StarId};

/// The frame a position, a velocity and a rotation are expressed in.
///
/// New frames are added at the end, so the numbers of the old ones never change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Frame {
    /// Fixed to a planet or a moon and turning with it: on foot, or a landed ship.
    Surface(BodyId),
    /// Centred on a planet or a moon and not turning with it: a ship inside its gravity well.
    Around(BodyId),
    /// Not turning, origin at a star: deep space.
    System(StarId),
}

impl Frame {
    /// Returns the star whose system the frame lies in. Players around the same star see each
    /// other at the full rate.
    pub fn star(&self) -> StarId {
        match self {
            Self::Surface(body) | Self::Around(body) => body.star,
            Self::System(star) => *star,
        }
    }
}

/// What a state record describes. New kinds are added at the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    /// The robot body of a player: a [`super::RobotBody`].
    Robot,
    /// The ship of a player: a [`super::ShipBody`].
    Ship,
    /// The companion of a player: a [`super::CompanionBody`].
    Companion,
}
