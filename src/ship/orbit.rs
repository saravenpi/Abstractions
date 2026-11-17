//! Orbit instruments: the highest and lowest points of the path the ship is on.

use bevy::math::DVec3;

/// Returns the apoapsis and periapsis radii in metres of a body at `pos` moving at `vel` around a
/// centre of gravitational parameter `mu`, or nothing when the path is not closed.
///
/// `pos` and `vel` are relative to the centre in a frame that does not turn.
pub fn apsides(pos: DVec3, vel: DVec3, mu: f64) -> Option<(f64, f64)> {
    let r = pos.length();
    let energy = vel.length_squared() * 0.5 - mu / r;
    if energy >= 0.0 || !energy.is_finite() {
        return None;
    }
    let axis = -mu / (2.0 * energy);
    let swept = pos.cross(vel).length_squared();
    let shape = (1.0 + 2.0 * energy * swept / (mu * mu)).max(0.0).sqrt();
    Some((axis * (1.0 + shape), axis * (1.0 - shape)))
}
