//! The speed tier of a route: how fast the drive may run on it.

use super::{CRUISE_CAP, GALAXY_CAP, JUMP_CAP, JUMP_DISTANCE};

/// How fast the drive may run on a route, set when the drive spools and kept until it stops, so
/// the cap never changes under a ship in warp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tier {
    /// Between the stars of a galaxy, up to [`JUMP_DISTANCE`]: the stars nearby are wells.
    #[default]
    Cruise,
    /// Farther inside a galaxy: blind to the stars on the way.
    Jump,
    /// To another galaxy: blind to the stars on the way.
    Galaxy,
}

impl Tier {
    /// Returns the tier of a route `distance` metres long, to another galaxy when `abroad`.
    pub fn of(distance: f64, abroad: bool) -> Self {
        match (abroad, distance > JUMP_DISTANCE) {
            (true, _) => Self::Galaxy,
            (false, true) => Self::Jump,
            (false, false) => Self::Cruise,
        }
    }

    /// Returns the fastest warp speed of the tier in metres per second.
    pub fn cap(self) -> f64 {
        match self {
            Self::Cruise => CRUISE_CAP,
            Self::Jump => JUMP_CAP,
            Self::Galaxy => GALAXY_CAP,
        }
    }

    /// Returns whether the route ignores the stars on the way and flies to a beacon.
    pub fn blind(self) -> bool {
        self != Self::Cruise
    }
}
