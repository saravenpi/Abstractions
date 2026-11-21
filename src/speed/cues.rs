//! What speed does to the picture and the sound, each a pure function of a real quantity.

use super::smoothstep;
use super::tune::{
    BUFFET, CONDENSE_GAIN, CONE_AIR, CONE_BAND, CONE_PEAK, CONTRAIL_AIR, CONTRAIL_DEPTH,
    CONTRAIL_LIFE, COOLING, DEW_SPREAD, MIXING, MOIST,
};

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

/// Returns how wet the air is `below` metres under the base of a cloud layer that covers the
/// share `cover` of the sky, from 0 to 1. The cloud base is where rising air saturates, so just
/// under it the air is nearly saturated, and a little lower the dew point lies more than the
/// cooling of [`COOLING`] behind. Inside the layer and above it, it is zero: [`moisture`] says
/// how wet the cloud is.
pub fn saturation(below: f32, cover: f32) -> f32 {
    if below < 0.0 {
        return 0.0;
    }
    let spread = below / DEW_SPREAD;
    cover.clamp(0.0, 1.0) * smoothstep(COOLING.0, COOLING.1, spread)
}

/// Returns how strongly the exhaust of an engine at `level` condenses into a contrail in air at
/// `cold` kelvin and `pressure` pascals, of `thickness` against the home sea level, from 0 to 1.
///
/// The Schmidt-Appleman criterion: the mixing line of the hot wet exhaust with the air must
/// cross water saturation, which happens under the threshold temperature
/// `226.69 + 9.43 ln(G - 0.053) + 0.72 ln^2(G - 0.053)` kelvin, with `G` the slope of
/// [`MIXING`] in pascals per kelvin (Schumann 1996). Near the ground that is about 247 K, at
/// 250 hPa about 231 K.
pub fn contrail(cold: f32, pressure: f32, thickness: f32, level: f32) -> f32 {
    let slope = (MIXING * f64::from(pressure)).max(0.06) - 0.053;
    let log = slope.ln();
    let threshold = (226.69 + 9.43 * log + 0.72 * log * log) as f32;
    let cool = smoothstep(threshold, threshold - CONTRAIL_DEPTH, cold);
    cool * smoothstep(CONTRAIL_AIR.0, CONTRAIL_AIR.1, thickness) * level.clamp(0.0, 1.0)
}

/// Returns the seconds a contrail lasts in air as wet as a sky under the cloud `cover` from 0
/// to 1: briefly in dry air, long where the air is wet enough to keep the ice.
pub fn persistence(cover: f32) -> f32 {
    let (dry, wet) = CONTRAIL_LIFE;
    dry + (wet - dry) * cover.clamp(0.0, 1.0)
}
