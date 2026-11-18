//! The neck: the head turns toward the view inside the reach of the neck and eases back to the
//! front beyond it, so a camera orbiting round to the face sees the face and never a snap.
//!
//! The yaw the view asks is wrapped to the half turn either side of the front, and the goal
//! fades to the front well before the back, so the head never turns through the back of the neck.

use bevy::prelude::*;

use super::clock::{ramp, spring};
use super::tune::{NECK_FADE, NECK_RATE, NECK_REACH};
use crate::player::wrap;

/// Returns where the head aims for a view `asked` relative to the body: the view inside the reach
/// of the neck, eased back to the front over [`NECK_FADE`] past its side reach.
pub fn reach(asked: Vec2) -> Vec2 {
    let yaw = wrap(asked.x);
    let keep = 1.0 - ramp(NECK_REACH.0, NECK_REACH.0 + NECK_FADE, yaw.abs());
    let held = Vec2::new(
        yaw.clamp(-NECK_REACH.0, NECK_REACH.0),
        asked.y.clamp(-NECK_REACH.1, NECK_REACH.1),
    );
    held * keep
}

/// Where the head looks and how fast it turns, relative to the body: `x` the yaw to the left,
/// `y` the pitch up.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Neck {
    /// Where the head looks.
    pub look: Vec2,
    /// How fast it turns, in radians per second.
    rate: Vec2,
}

impl Neck {
    /// Turns the head for `dt` seconds toward the reach of the view `asked`, critically damped,
    /// and returns where it looks.
    pub fn step(&mut self, asked: Vec2, dt: f32) -> Vec2 {
        let goal = reach(asked);
        let gap = self.look - goal;
        let (x, rate_x) = spring(gap.x, self.rate.x, NECK_RATE, dt);
        let (y, rate_y) = spring(gap.y, self.rate.y, NECK_RATE, dt);
        self.look = goal + Vec2::new(x, y);
        self.rate = Vec2::new(rate_x, rate_y);
        self.look
    }
}
