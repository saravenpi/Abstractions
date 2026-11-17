//! The body on a sphere: its upright frame, its yaw around the local up and the camera pitch.
//!
//! The upright frame at a point is render space turned by the smallest rotation that takes
//! render +Y to the local up. The controller thinks in that frame, where +Y is always up.
//! The yaw is stored, never read back from the orientation, so the heading cannot drift.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

/// Returns the rotation from the upright frame at a point whose local vertical is `up` to render space.
///
/// The arc is taken in 64 bits: in 32 bits it is the identity below 0.04 degrees, which would
/// leave the body untilted for the first metres around the anchored spot.
pub(super) fn tilt(up: Vec3) -> Quat {
    DQuat::from_rotation_arc(DVec3::Y, up.as_dvec3().normalize()).as_quat()
}

/// Returns the orientation of a body standing along `up` and turned by `yaw` around it.
pub(super) fn facing(up: Vec3, yaw: f32) -> Quat {
    tilt(up) * Quat::from_rotation_y(yaw)
}

/// Returns the yaw around `up` that looks along the render direction `toward`.
pub(super) fn yaw_toward(toward: Vec3, up: Vec3) -> f32 {
    let local = tilt(up).inverse() * toward;
    (-local.x).atan2(-local.z)
}

/// Returns the pitch of the camera, a child of the body that only looks up and down.
pub(super) fn pitch_of(camera: &Transform) -> f32 {
    camera.rotation.to_euler(EulerRot::YXZ).1
}
