//! A rotation on a second-order spring, for bodies that tilt into their way.

use bevy::math::{Quat, Vec3};

use super::spring::Spring;

/// A rotation that follows its target through a [`Spring`], with its angular velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turn {
    /// Current rotation.
    pub at: Quat,
    /// Angular velocity as a scaled axis, radians per second.
    pub vel: Vec3,
    /// Target of the last step.
    last: Quat,
}

/// Returns the shortest rotation from `from` to `to` as a scaled axis.
fn between(from: Quat, to: Quat) -> Vec3 {
    let delta = to * from.inverse();
    let delta = if delta.w < 0.0 { -delta } else { delta };
    delta.to_scaled_axis()
}

impl Turn {
    /// Returns a rotation resting at `at`.
    pub fn new(at: Quat) -> Self {
        Self {
            at,
            vel: Vec3::ZERO,
            last: at,
        }
    }

    /// Turns toward `target` by `dt` seconds of `spring` and returns the rotation.
    pub fn step(&mut self, spring: &Spring, target: Quat, dt: f32) -> Quat {
        if dt <= 0.0 {
            return self.at;
        }
        let pull = between(self.last, target) / dt;
        self.last = target;
        self.at = (Quat::from_scaled_axis(self.vel * dt) * self.at).normalize();
        self.vel += spring.accel(between(self.at, target), pull, self.vel, dt);
        self.at
    }
}
