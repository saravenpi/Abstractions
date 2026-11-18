//! The neck: the head turns toward what the companion looks at, and the body follows it.
//!
//! Both turns are critically damped springs with a top speed, so the companion never snaps
//! round like a turret. The head leads within the reach of its neck; the body trails behind
//! and is pushed along once the head would turn past that reach.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::spring::step;
use super::tune::{BODY_RATE, BODY_SPRING, HEAD_RATE, HEAD_SPRING, NECK};

/// Steepest the body itself points up or down, in radians, so it never lies on its back.
const BODY_PITCH: f32 = 1.2;

/// How the head is turned from the body, and how fast both turn.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Neck {
    /// Turn of the head from the body in radians: x to the left about the body's up, y upward.
    pub head: Vec2,
    /// Turn rate of the head in radians per second, in the same angles.
    spin: Vec2,
    /// Turn rate of the body in radians per second, in the same angles.
    sway: Vec2,
}

/// Returns the turn to the left and up of a direction given in the body's own axes, whose front is -Z.
fn angles(local: Vec3) -> Vec2 {
    let flat = Vec2::new(local.x, local.z).length();
    Vec2::new((-local.x).atan2(-local.z), local.y.atan2(flat))
}

/// Returns `angle` brought back between -pi and pi.
fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Advances a spring toward a target `gap` away at `rate` with velocity `vel`, by at most
/// `top` radians per second, and returns the move and the new velocity.
fn chase(gap: Vec2, vel: Vec2, (rate, top): (f32, f32), dt: f32) -> (Vec2, Vec2) {
    let (left, vel) = step(-gap.extend(0.0), vel.extend(0.0), rate, dt);
    let moved = (left.truncate() + gap).clamp_length_max(top * dt);
    (moved, vel.truncate().clamp_length_max(top))
}

/// Returns `rot` turned by `turn` (to the left about `up`, then upward), kept upright along `up`.
fn swing(rot: Quat, turn: Vec2, up: Vec3) -> Quat {
    let ahead = Quat::from_axis_angle(up, turn.x) * (rot * Vec3::NEG_Z);
    let right = ahead.cross(up).normalize_or(rot * Vec3::X);
    let raised = Quat::from_axis_angle(right, turn.y) * ahead;
    let rise = raised.dot(up).clamp(-1.0, 1.0).asin();
    let level = (raised - up * raised.dot(up)).normalize_or(ahead);
    let pitch = rise.clamp(-BODY_PITCH, BODY_PITCH);
    let dir = level * pitch.cos() + up * pitch.sin();
    Transform::IDENTITY.looking_to(dir, up).rotation
}

impl Neck {
    /// Returns the rotation of the head in the body's axes.
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.head.x) * Quat::from_rotation_x(self.head.y)
    }

    /// Turns the head toward `dir` and the body after it, by `dt` seconds, and returns the new
    /// rotation of the body, which was `rot`, upright along `up`.
    pub fn turn(&mut self, rot: Quat, dir: Vec3, up: Vec3, dt: f32) -> Quat {
        let Some(dir) = dir.try_normalize() else {
            return rot;
        };
        let want = angles(rot.inverse() * dir);
        let gap = Vec2::new(wrap(want.x - self.head.x), want.y - self.head.y);
        let (moved, spin) = chase(gap, self.spin, (HEAD_SPRING, HEAD_RATE), dt);
        let head = self.head + moved;
        let (mut body, sway) = chase(head, self.sway, (BODY_SPRING, BODY_RATE), dt);
        let reach = Vec2::from(NECK);
        let rest = head - body;
        body += rest - rest.clamp(-reach, reach);
        (self.head, self.spin, self.sway) = (head - body, spin, sway);
        swing(rot, body, up)
    }
}
