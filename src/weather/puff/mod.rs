//! The fine cloud: the puffs the cloud shells draw, as a function of a point of the shell.
//!
//! This is the Rust twin of `render/cloud_field.wesl`. [`hash31`], [`space_noise`], [`turned`],
//! [`threshold`] and the functions of `seen` and `graze` mirror the shader functions of the same names, and
//! the constants their constants of the same names. The hash is whole-number math, so both sides
//! agree up to float rounding. [`WIND`] and [`SHEAR`] live here alone: the shader is handed the
//! turn [`drift`] returns.
//!
//! The puffs ride the planet: they are laid in axes fixed to its ground and turn east round its
//! spin axis at the speed of the wind, a rotation whose angle is worked out in 64 bits and is
//! periodic, so no time of play and no move of the render origin makes them jump. The finer
//! octaves turn a little faster than the broad one, so the puffs change shape as they go.

#[cfg(test)]
mod graze;
mod seen;

use bevy::math::{Vec3, Vec4};

#[cfg(test)]
pub use graze::{CLAMP, CUT, GRAZE, MEAN, SPREAD, TAIL, TALL, drawn};
pub use seen::density_at;
#[cfg(test)]
pub use seen::{SHARP, density, fbm_seen};

/// Width in metres of one cell of the cloud noise: a puff is one to a few cells across, a cumulus
/// of a few kilometres. At 450 m a puff seen from the ground under a high cloud band was a few
/// pixels wide overhead and under one at the horizon, a sky of dashes.
pub const CELL: f32 = 1800.0;

/// Speed in metres per second at which the puffs drift east at the equator of their shell: the
/// speed the broad weather of the field drifts at, so the puffs ride their cover. Unit: m/s.
pub const WIND: f64 = 4.0;

/// Share of [`WIND`] the finer octaves of the noise move by on top of it, so a puff changes shape
/// as its detail slides over its body: 1 m/s, which remakes a detail cell of 900 m in a quarter
/// of an hour, about the life of a cumulus. Unit: share.
pub const SHEAR: f64 = 0.25;

/// Width of the soft edge of a puff, in noise value.
pub const EDGE: f32 = 0.12;

/// Noise value a puff starts at under no cover and under full cover. The noise has a mean near
/// 0.47, so half a cover puts cloud over about a quarter of the sky, and a full cover leaves only
/// rare breaks in the deck. At 0.4 for a full cover two fifths of an overcast sky were gaps.
pub const THRESHOLD: (f32, f32) = (0.72, 0.25);

/// Density a puff must reach before it shows at all, and from which it shows in full, under a clear
/// sky: a lone tip of the noise barely over the threshold is no cloud. The cut eases off as the cover
/// grows and is gone under an overcast sky, whose deck is whole. Twin of `LONE` in the shader.
pub const LONE: (f32, f32) = (0.3, 0.8);

/// How many times finer each octave of the noise is than the one before.
pub const OCTAVE_SCALE: f32 = 2.03;

/// Cells each octave of the noise is shifted by from the one before.
pub const OCTAVE_SHIFT: Vec3 = Vec3::new(17.1, 9.2, 4.7);

/// Octaves of the noise a puff is cut from.
pub const OCTAVES: u32 = 4;

/// A point of the noise of the puffs: where the broad octave reads it, and where the finer
/// octaves do, which have slid on by [`SHEAR`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// The point the first octave reads, in cells.
    pub base: Vec3,
    /// The point the finer octaves start from, in cells.
    pub fine: Vec3,
}

impl Spot {
    /// Returns the spot whose octaves all read the point `p`, as on a field that has not moved.
    #[cfg(test)]
    pub fn same(p: Vec3) -> Self {
        Self { base: p, fine: p }
    }
}

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

/// Returns the noise value a puff starts at under the broad `cover`. Twin of `threshold`.
pub fn threshold(cover: f32) -> f32 {
    THRESHOLD.0 * (1.0 - cover) + THRESHOLD.1 * cover
}

/// Returns the turn of the puffs on a shell `cells` noise cells in radius at `seconds` of
/// universe time: the cosine and the sine of the angle the broad octave has turned east by round
/// the spin axis, then those of the finer octaves. The angles are worked out in 64 bits and taken
/// round the circle, so they hold to a fraction of a metre after any time of play.
pub fn drift(seconds: f64, cells: f32) -> Vec4 {
    let rate = WIND / (f64::from(cells.max(1.0)) * f64::from(CELL));
    let angle = |share: f64| (rate * share * seconds).rem_euclid(std::f64::consts::TAU);
    let (base, fine) = (angle(1.0), angle(1.0 + SHEAR));
    Vec4::new(
        base.cos() as f32,
        base.sin() as f32,
        fine.cos() as f32,
        fine.sin() as f32,
    )
}

/// Returns `v` turned back round the spin axis, +Y, by the angle of cosine `c` and sine `s`:
/// where the noise that now lies over `v` was before it drifted east. Twin of `turned`.
pub fn turned(v: Vec3, c: f32, s: f32) -> Vec3 {
    Vec3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}

/// Returns the spot of the noise over the planet-fixed point `p` of the shell, in cells, under
/// the turn `drift`. Twin of `spot_at`.
pub fn spot_at(p: Vec3, drift: Vec4) -> Spot {
    Spot {
        base: turned(p, drift.x, drift.y),
        fine: turned(p, drift.z, drift.w),
    }
}
