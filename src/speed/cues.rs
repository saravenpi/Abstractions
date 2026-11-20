//! What speed does to the picture and the sound, each a pure function of a real quantity.

use super::tune::{BUFFET, CONDENSE_GAIN, CONE_AIR, CONE_BAND, CONE_PEAK, MOIST};
use crate::render::smoothstep;

/// Returns how hard the air shakes a body in flight, from 0 to 1, for a dynamic pressure in pascals.
///
/// It grows with `1/2 rho v^2`, not with speed alone: it is zero in vacuum at any speed and in thin
/// air up high, and full in thick air at speed.
pub fn buffet(pressure: f32) -> f32 {
    smoothstep(BUFFET.0, BUFFET.1, pressure)
}

/// Returns how wet the air is, from 0 (dry) to 1 (in cloud or in rain): the `depth` in the cloud layer
/// from 0 to 1 under the `cloud` cover the weather puts in it, or the `rain`, whichever is more.
pub fn moisture(depth: f32, cloud: f32, rain: f32) -> f32 {
    (depth * cloud).max(rain).clamp(0.0, 1.0)
}

/// Returns how much the low pressure of a wingtip vortex condenses, from 0 in dry air to 1 in cloud or
/// rain, for the wetness of the air from 0 to 1.
pub fn condensation(moisture: f32) -> f32 {
    smoothstep(MOIST.0, MOIST.1, moisture)
}

/// Returns the factor a wingtip trail is brighter by, from 1 in dry air.
pub fn vortex(moisture: f32) -> f32 {
    1.0 + CONDENSE_GAIN * condensation(moisture)
}

/// Returns how strongly a vapour cone shows around a ship at `mach` in air of `thickness` (the
/// density against the home sea level, from 0 to 1) and wetness `moisture` from 0 to 1.
///
/// The cone shows in a band around Mach 1 only, in air that is wet and dense enough to condense.
pub fn cone(mach: f32, thickness: f32, moisture: f32) -> f32 {
    let band =
        smoothstep(CONE_BAND.0, CONE_BAND.1, mach) * (1.0 - smoothstep(1.0, CONE_PEAK, mach));
    band * smoothstep(CONE_AIR.0, CONE_AIR.1, thickness) * condensation(moisture)
}
