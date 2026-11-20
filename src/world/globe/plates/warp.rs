//! The warp: a smooth displacement of the direction a plate is looked up at, so that plate edges bend
//! instead of following great circles.

use bevy::math::DVec3;

use super::tune::{WARP, WARP_GRAIN};
use crate::world::islands::gradient;

/// Where the warp noise is read for each axis of the displacement, in noise units.
const SHIFT: [DVec3; 3] = [
    DVec3::ZERO,
    DVec3::new(17.3, -41.9, 8.6),
    DVec3::new(-29.4, 12.7, 33.1),
];

/// Returns unit direction `dir` displaced by up to [`WARP`] radians along each axis, back on the unit
/// sphere. The same `seed` and direction always give the same result.
pub fn warped(seed: u32, dir: DVec3) -> DVec3 {
    let q = dir * WARP_GRAIN;
    let push = |k: usize| gradient(seed.wrapping_add(k as u32), (q + SHIFT[k]).to_array());
    (dir + DVec3::new(push(0), push(1), push(2)) * WARP).normalize()
}
