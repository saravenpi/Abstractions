//! The loose strap beside the seat: a pendulum on the frame of the seat, which hangs along
//! the pull the crew feels and swings after every change of it.

use bevy::prelude::*;

use super::tune::{SLACK, SPRING_STEP, SWING};

/// The state of the strap: how far it hangs off the vertical along x and z, as the slope of its
/// line, and how fast that slope changes per second.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Strap {
    /// Slope of the strap off the vertical along x and z.
    pub tilt: Vec2,
    /// Change of the slope per second.
    pub rate: Vec2,
}

/// Returns the slope a strap settles at under the acceleration `accel` the crew feels, in body
/// axes: it hangs along the pull of the frame, which is the opposite of that acceleration.
/// Under [`SLACK`] of vertical pull it floats near upright, and it is held to a slope of one.
pub fn hang(accel: Vec3) -> Vec2 {
    let apart = -accel;
    (Vec2::new(apart.x, apart.z) / (-apart.y).max(SLACK)).clamp_length_max(1.0)
}

impl Strap {
    /// Returns the strap `dt` seconds later under the acceleration `accel`: a damped spring
    /// toward the slope it hangs at, stepped in pieces no longer than [`SPRING_STEP`].
    pub fn stepped(self, accel: Vec3, dt: f32) -> Strap {
        let (pulse, damping) = SWING;
        let dt = dt.min(SPRING_STEP);
        let pull = -(pulse * pulse) * (self.tilt - hang(accel)) - 2.0 * damping * pulse * self.rate;
        let rate = self.rate + pull * dt;
        Strap {
            tilt: self.tilt + rate * dt,
            rate,
        }
    }

    /// Returns the turn of the strap on its hinge: its line hangs along the slope.
    pub fn turn(&self) -> Quat {
        let line = Vec3::new(self.tilt.x, -1.0, self.tilt.y).normalize();
        Quat::from_rotation_arc(Vec3::NEG_Y, line)
    }
}
