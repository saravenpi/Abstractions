//! The fine cloud: the puffs the cloud shells draw, as a function of a point of the shell.
//!
//! This is the Rust twin of `render/cloud_field.wesl`. [`hash31`], [`space_noise`], [`fbm`],
//! [`threshold`] and [`density`] mirror the shader functions of the same names, and [`CELL`],
//! [`EDGE`], [`THRESHOLD`], [`OCTAVE_SCALE`] and [`OCTAVE_SHIFT`] its constants. The hash is
//! whole-number math, so both sides agree up to float rounding. [`WIND`] lives here alone: the
//! shader is handed the offset [`drift`] returns.

use bevy::math::{DVec3, Vec3};

use crate::render::smoothstep;

/// Width in metres of one cell of the cloud noise.
pub const CELL: f32 = 450.0;

/// Cells of the cloud noise the puffs drift by in one second of universe time.
pub const WIND: Vec3 = Vec3::new(0.011, 0.0, 0.0045);

/// Width of the soft edge of a puff, in noise value.
pub const EDGE: f32 = 0.12;

/// Noise value a puff starts at under no cover and under full cover.
pub const THRESHOLD: (f32, f32) = (0.72, 0.4);

/// Density a puff must reach before it shows at all, and from which it shows in full: a lone tip
/// of the noise barely over the threshold is no cloud. Twin of `LONE` in the shader.
pub const LONE: (f32, f32) = (0.3, 0.8);

/// How many times finer each octave of the noise is than the one before.
pub const OCTAVE_SCALE: f32 = 2.03;

/// Cells each octave of the noise is shifted by from the one before.
pub const OCTAVE_SHIFT: Vec3 = Vec3::new(17.1, 9.2, 4.7);

/// Octaves of the noise a puff is cut from.
pub const OCTAVES: u32 = 4;

/// Seconds of universe time after which the drift starts over, so that the offset stays small
/// enough for 32-bit floats to hold to a few centimetres: about a day and a half.
pub const WRAP: f64 = 131_072.0;

/// Returns the value from 0 to 1 of the whole point `p`. Twin of `hash31` in the shader.
pub fn hash31(p: Vec3) -> f32 {
    let q = p.as_ivec3();
    let (x, y, z) = (q.x as u32, q.y as u32, q.z as u32);
    let mut h = x
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add(y.wrapping_mul(0x1656_67b1))
        .wrapping_add(z.wrapping_mul(0x9e37_79b1));
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / 16_777_216.0
}

/// Returns the smooth noise at `p`, from 0 to 1. Twin of `space_noise` in the shader.
pub fn space_noise(p: Vec3) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let mix = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let at = |x: f32, y: f32, z: f32| hash31(i + Vec3::new(x, y, z));
    let row = |y: f32, z: f32| mix(at(0.0, y, z), at(1.0, y, z), u.x);
    let sheet = |z: f32| mix(row(0.0, z), row(1.0, z), u.y);
    mix(sheet(0.0), sheet(1.0), u.z)
}

/// Returns `octaves` octaves of the noise at `p`. Twin of `fbm` in the shader.
pub fn fbm(p: Vec3, octaves: u32) -> f32 {
    let (mut total, mut amplitude, mut q) = (0.0, 0.5, p);
    for _ in 0..octaves {
        total += amplitude * space_noise(q);
        q = q * OCTAVE_SCALE + OCTAVE_SHIFT;
        amplitude *= 0.5;
    }
    total
}

/// Returns the noise value a puff starts at under the broad `cover`. Twin of `threshold`.
pub fn threshold(cover: f32) -> f32 {
    THRESHOLD.0 * (1.0 - cover) + THRESHOLD.1 * cover
}

/// Returns the cells the puffs have drifted by at `seconds` of universe time: the offset both
/// the shaders and [`density_at`] add to a point of the shell. It is worked out in 64 bits and
/// starts over every [`WRAP`] seconds.
pub fn drift(seconds: f64) -> Vec3 {
    (WIND.as_dvec3() * seconds.rem_euclid(WRAP)).as_vec3()
}

/// Returns how much cloud there is at the point `spot` of the shell, in cells of the noise
/// with the drift added, under the broad `cover`: 0 in clear air, 1 inside a puff. Twin of
/// `density` in the shader, without the thickening the shader adds at the limb.
pub fn density(spot: Vec3, cover: f32) -> f32 {
    let from = threshold(cover);
    let dense = smoothstep(from, from + EDGE, fbm(spot, OCTAVES));
    dense * smoothstep(LONE.0, LONE.1, dense)
}

/// Returns how much cloud there is in the planet-fixed unit direction `dir` of a shell
/// `cells` noise cells in radius, at `seconds` of universe time, under the broad `cover`.
pub fn density_at(dir: DVec3, cells: f32, seconds: f64, cover: f32) -> f32 {
    density(dir.as_vec3() * cells + drift(seconds), cover)
}
