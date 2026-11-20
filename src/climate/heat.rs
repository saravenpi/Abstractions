//! Heat: latitude from the spin axis, then the mean, the pole to equator shape, the season and
//! the cooling with height.
//!
//! Shape and lapse follow the climate notes of davudk, procedural-world-generation,
//! <https://github.com/davudk/procedural-world-generation/blob/main/docs/research/03-climate-biomes.md>.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::math::Vec3;

use super::{Setting, Spot};
use crate::universe::STANDARD_GRAVITY;

/// Kelvin between the poles and the equator at home air pressure.
const HOME_GAP: f32 = 40.0;

/// Stretch of the latitude shape, which puts the equator 16 K over the mean and the poles 32 K under.
const SHAPE: f32 = 1.2;

/// Equal-area mean of the squared sine of the latitude: taking it off keeps the planet mean.
const MEAN_SQUARE: f32 = 1.0 / 3.0;

/// Air pressure under which thicker air stops evening out the gap: the gap is at most twice the home one.
const THIN_AIR: f32 = 0.25;

/// Kelvin lost per metre of height at home gravity and air.
const LAPSE: f32 = 6.5e-3;

/// Kelvin a tidally locked moon is over its mean under its planet, and under it on the far side.
const NEAR_SIDE: f32 = 15.0;

/// Largest shift of the latitude by the warp noise, in radians: 3 degrees.
const WARP: f32 = 3.0 * std::f32::consts::PI / 180.0;

/// Waves of the warp per radian of arc: a wavelength of a quarter of the circumference.
const WAVES: f32 = 4.0;

/// Unit axes of the three warp waves, spread apart and none along the spin axis.
const AXES: [Vec3; 3] = [
    Vec3::new(0.8, 0.36, 0.48),
    Vec3::new(-0.48, 0.6, 0.64),
    Vec3::new(0.36, -0.48, 0.8),
];

/// Returns the bits of `seed` mixed by splitmix64, so that close seeds give unrelated values.
fn mix(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Returns the warp of the latitude at `dir`, from -[`WARP`] to [`WARP`]: three slow waves along
/// fixed axes with seeded phases, so the climate belts wander instead of following the parallels.
fn warp(seed: u64, dir: Vec3) -> f32 {
    let bits = mix(seed);
    let phase = |k: usize| (bits >> (16 * k)) as u16 as f32 * (TAU / 65_536.0);
    let wave = |k: usize| (WAVES * AXES[k].dot(dir) + phase(k)).sin();
    WARP * (wave(0) + wave(1) + wave(2)) / 3.0
}

/// Returns the latitude of `dir` in radians, measured from the equator of the spin axis +Y and
/// warped by a few degrees of slow noise.
pub(super) fn latitude(setting: &Setting, dir: Vec3) -> f32 {
    let lat = dir.y.clamp(-1.0, 1.0).asin() + warp(setting.seed, dir);
    lat.clamp(-FRAC_PI_2, FRAC_PI_2)
}

/// Returns the kelvin between the poles and the equator: 40 at home air, less in thick air,
/// which carries heat to the poles, and at most twice as much in thin air or none.
fn gap(setting: &Setting) -> f32 {
    HOME_GAP / setting.pressure.max(THIN_AIR).sqrt()
}

/// Returns the air temperature in kelvin at `spot`, at latitude `lat`, at `season` of the year.
///
/// The latitude shape is `gap * (1/3 - sin^2 lat) * 1.2`, whose equal-area mean is zero so the
/// planet mean stays the record mean. A tidally locked moon has no such shape: its near side is
/// 15 K warmer, its far side 15 K colder. The season adds `gap * sin(tilt) * sin(lat) *
/// cos(2 pi season)`, opposite in the two hemispheres, and height takes 6.5 K per km at standard
/// gravity, scaled by the gravity, above the liquid surface only.
pub(super) fn temperature(setting: &Setting, spot: &Spot, lat: f32, season: f32) -> f32 {
    let gap = gap(setting);
    let sin = lat.sin();
    let shape = match setting.near {
        Some(near) => NEAR_SIDE * spot.dir.as_vec3().dot(near),
        None => gap * (MEAN_SQUARE - sin * sin) * SHAPE,
    };
    let swing = gap * setting.tilt.sin() * sin * (TAU * season).cos();
    let lapse = LAPSE * setting.gravity / STANDARD_GRAVITY as f32 * spot.altitude.max(0.0);
    setting.temperature + shape + swing - lapse
}
