//! The head look-at: the head turns toward a point, as far as its neck lets it.

use bevy::math::{EulerRot, Quat, Vec2, Vec3};

/// Largest turn of the head to either side, in radians.
const YAW: f32 = 1.2;

/// Largest tilt of the head up or down, in radians.
const PITCH: f32 = 0.6;

/// Returns the yaw (to the left) and the pitch (up) that turn a head at `head` on a body turned by
/// `turn` toward `target`, clamped to what a neck allows. No target looks straight ahead.
pub fn aim(target: Option<Vec3>, head: Vec3, turn: Quat) -> Vec2 {
    let Some(target) = target else {
        return Vec2::ZERO;
    };
    let d = turn.inverse() * (target - head);
    let yaw = (-d.x).atan2(-d.z);
    let pitch = d.y.atan2(d.x.hypot(d.z));
    Vec2::new(yaw.clamp(-YAW, YAW), pitch.clamp(-PITCH, PITCH))
}

/// Returns the rotation, in the frame `turn` is the body's rotation in, that turns a head by `look`.
pub fn tilt(look: Vec2, turn: Quat) -> Quat {
    turn * Quat::from_euler(EulerRot::YXZ, look.x, look.y, 0.0) * turn.inverse()
}
