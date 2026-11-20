//! The attitude of the ship against its surroundings, as the displays of the dashboard draw it.

use bevy::prelude::*;

/// Returns the pitch up and the roll to the right of a ship, in radians, from the local up in
/// body axes: the nose is along -Z and the right wing along +X. Nothing for an up that has no
/// direction.
pub fn attitude(up: Vec3) -> Option<Vec2> {
    let up = up.try_normalize()?;
    let pitch = (-up.z).clamp(-1.0, 1.0).asin();
    Some(Vec2::new(pitch, (-up.x).atan2(up.y)))
}

/// Returns the point of a scope that `direction` falls on, in body axes, for a scope of
/// `radius` pixels: the nose is at the centre and straight behind is on the rim, the way
/// there is the angle off the nose. The point is in pixels, x to the right and y up.
pub fn project(direction: Vec3, radius: f32) -> IVec2 {
    let off = (-direction.z).clamp(-1.0, 1.0).acos();
    let flat = Vec2::new(direction.x, direction.y).normalize_or_zero();
    (flat * off / std::f32::consts::PI * radius)
        .round()
        .as_ivec2()
}

/// Returns `angle` in whole steps of `step` radians.
pub fn steps(angle: f32, step: f32) -> i32 {
    (angle / step).round() as i32
}
