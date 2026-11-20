//! The shape of the flight effects: which way they stream.

use bevy::prelude::*;

/// Returns where the effects stream from: along `velocity`, or along `ahead`, the nose of the
/// ship, without motion and from the first moment the drive spools. In a star frame the
/// velocity still carries the orbit of the planet the ship left, which swept the warp streaks
/// sideways while the drive spooled.
pub fn stream_heading(velocity: Vec3, ahead: Vec3, warp: f32) -> Vec3 {
    if warp > 0.0 {
        return ahead;
    }
    velocity.try_normalize().unwrap_or(ahead)
}
