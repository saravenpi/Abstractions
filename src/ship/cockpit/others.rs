//! The other controls: the gear lever, the warp switch and the rudder pedals.

use bevy::prelude::*;

use super::controls::{Levers, throw};
use super::tune::{PEDAL_REACH, THROW};

impl Levers {
    /// Returns the turn of the gear lever in its hinge: back with the gear up, forward with
    /// it down.
    pub fn gear_turn(&self) -> Quat {
        throw(THROW.0, self.gear)
    }

    /// Returns the turn of the warp switch in its hinge: back with the drive off, upright
    /// while it spools, forward when it is engaged.
    pub fn switch_turn(&self) -> Quat {
        throw(THROW.1, self.warp)
    }

    /// Returns how far each pedal is pushed, from -1 to 1: the left pedal goes forward for a
    /// yaw to the left, and the right pedal goes back by as much.
    pub fn pedals(&self) -> [f32; 2] {
        let yaw = self.stick.y.clamp(-1.0, 1.0);
        [yaw, -yaw]
    }

    /// Returns the turn of the pedal of `side`, 0 for the left and 1 for the right: the toe
    /// goes down when the pedal is pushed forward.
    pub fn pedal_turn(&self, side: usize) -> Quat {
        Quat::from_rotation_x(-PEDAL_REACH * self.pedals()[side.min(1)])
    }
}
