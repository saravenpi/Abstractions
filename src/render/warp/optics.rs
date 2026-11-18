//! What speed does to the sky, as special relativity has it: light from ahead of a fast observer
//! crowds toward the point it flies at, turns bluer and brighter, and light from behind turns
//! redder and dimmer. `starfield.wesl` applies these rules to the stars; here they are tested
//! and applied to the bodies.

use bevy::math::DVec3;

use crate::render::celestial::smoothstep;

/// Radii from a body inside which it rides in the warp bubble and is seen as it is, and beyond
/// which it is seen through the whole speed of the ship. The drive carries the bubble along
/// with a body between the same radii.
pub const CARRIED: (f32, f32) = (50.0, 100.0);

/// Returns the Doppler factor of light from a point whose true angle from the heading has the
/// cosine `cos`: above 1 ahead, where light turns bluer, below 1 behind. A patch of sky there
/// is seen smaller by the same factor.
pub fn doppler(cos: f32, beta: f32) -> f32 {
    (1.0 + beta * cos) / (1.0 - beta * beta).max(1.0e-6).sqrt()
}

/// Returns the unit vector toward which the true direction `dir` is seen at the speed `beta`,
/// a share of the speed of light, along the unit vector `heading`.
///
/// At rest it is `dir`. The angle from the heading shrinks as the speed rises, and the points
/// straight ahead and straight behind stay where they are. It works in 64 bits and from the
/// sine of the angle, so a body a hair off the heading stays a hair off it instead of jumping
/// by the grain of a 32-bit cosine.
pub fn bend(dir: DVec3, heading: DVec3, beta: f32) -> DVec3 {
    let (cos, beta) = (dir.dot(heading), f64::from(beta));
    let out = dir - heading * cos;
    let shrink = (1.0 - beta * beta).max(0.0).sqrt() / (1.0 + beta * cos);
    (heading * ((cos + beta) / (1.0 + beta * cos)) + out * shrink).normalize_or(dir)
}

/// Returns how much of the speed of the ship a body `radii` of its own radii away is seen
/// through: none while it rides in the bubble, all of it in the open.
pub fn bubble(radii: f32) -> f32 {
    smoothstep(CARRIED.0, CARRIED.1, radii)
}
