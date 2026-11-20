//! Touching down with more than the knees: how the body takes a landing, chosen from the impact
//! the gear recorded, and the poses of the roll, the tumble and the skid.
//!
//! The roll and the tumble turn the whole trunk about the ground whole turns, driven by the
//! distance the body really travels. That turn lives in the clock and is added on top of the
//! blend, like the pitch of a swimming body, so the weights of the modes never spin it backwards.

pub mod floor;
pub mod roll;
mod skid;
pub mod tune;

pub use floor::grounded;
pub use roll::{roll, spin};
pub use skid::{lie, skid};

use bevy::prelude::*;

use super::Mode;
use crate::gear::Mode as Flight;
use tune::{HANDS, ROLL_AHEAD, TUMBLE_LOST};

/// How a body takes a touchdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touch {
    /// The knees absorb it and the arms balance.
    Soft,
    /// A deep crouch with one or both hands on the ground.
    Hard,
    /// A roll over the shoulder along the way, forward or backward.
    Ahead,
    /// A roll over the side.
    Side,
    /// A skid along the ground, on the feet or on the belly.
    Skid,
    /// A tumble: the trunk keeps turning and the limbs flail.
    Tumble,
}

impl Touch {
    /// Returns the mode that poses the body for this touchdown.
    pub fn mode(self) -> Mode {
        match self {
            Touch::Soft | Touch::Hard => Mode::Land,
            Touch::Ahead | Touch::Side | Touch::Tumble => Mode::Roll,
            Touch::Skid => Mode::Skid,
        }
    }
}

/// Returns how a body takes a touchdown the gear handles as `flight`, having come in at `struck`
/// in the frame of the body and lost `lost` meters per second into the ground, with a landing
/// as hard as `hard` from 0 to 1.
///
/// A landing roll of the gear is a tumble after a crash, a crouch on the spot when it barely
/// moves along the ground, and otherwise a roll over the side it mostly travels to.
pub fn touch(flight: Flight, struck: Vec3, lost: f32, hard: f32) -> Touch {
    let flat = struck.with_y(0.0);
    match flight {
        Flight::Roll if lost >= TUMBLE_LOST.1 => Touch::Tumble,
        Flight::Roll if flat.length() < ROLL_AHEAD => Touch::Hard,
        Flight::Roll if flat.x.abs() > flat.z.abs() => Touch::Side,
        Flight::Roll => Touch::Ahead,
        Flight::Slide => Touch::Skid,
        _ if hard >= HANDS.0 => Touch::Hard,
        _ => Touch::Soft,
    }
}
