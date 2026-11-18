//! Slow seeded noise over place and time: smooth lattice noise in space whose values ease from
//! one knot of time to the next, so the weather is a place, drifts over minutes and comes back
//! the same for the same seed, place and second.

use bevy::math::DVec3;

use crate::audio::Rand;

/// Seconds between two knots of the broad swell and of the faster ripple on top of it.
const PERIODS: [f64; 2] = [300.0, 97.0];

/// Weight of the faster ripple against the broad swell.
const RIPPLE: f32 = 0.35;

/// How many times finer than the swell the ripple is in space.
const FINER: f64 = 2.0;

/// How much the noise is stretched around its middle: smoothing over place narrows it, and
/// this gives the weather back the range it had when it was noise in time alone.
const CONTRAST: f32 = 1.5;

/// Returns the value of knot `index` of channel `channel` on the planet of `seed`, from 0 to 1.
pub fn knot(seed: u64, channel: u64, index: i64) -> f32 {
    stream(seed, channel, index).unit()
}

/// Returns the generator of knot `index` of channel `channel` on the planet of `seed`.
fn stream(seed: u64, channel: u64, index: i64) -> Rand {
    Rand::new(
        seed ^ channel.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        index as u64,
    )
}

/// Returns the value from 0 to 1 the lattice of `key` holds at the whole point `cell`.
fn corner(key: u64, cell: [i64; 3]) -> f32 {
    let [x, y, z] = cell.map(|c| c as u64);
    let mut h = key
        ^ x.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ y.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ z.wrapping_mul(0x1656_67B1_9E37_79F9);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((h ^ (h >> 31)) >> 40) as f32 / 16_777_216.0
}

/// Returns the smooth noise of the lattice of `key` at `at`, from 0 to 1: the eight corners of
/// the cell around it, eased along each axis.
fn lattice(key: u64, at: DVec3) -> f32 {
    let floor = at.floor();
    let u = (at - floor).as_vec3();
    let ease = u * u * (3.0 - 2.0 * u);
    let base = [floor.x as i64, floor.y as i64, floor.z as i64];
    let value = |dx: i64, dy: i64, dz: i64| corner(key, [base[0] + dx, base[1] + dy, base[2] + dz]);
    let along_x =
        |dy: i64, dz: i64| value(0, dy, dz) + (value(1, dy, dz) - value(0, dy, dz)) * ease.x;
    let along_y = |dz: i64| along_x(0, dz) + (along_x(1, dz) - along_x(0, dz)) * ease.y;
    along_y(0) + (along_y(1) - along_y(0)) * ease.z
}

/// Returns one octave of channel `channel` at `at` and `seconds`: one lattice per knot of
/// time, `period` seconds apart, eased from one to the next.
fn octave(seed: u64, channel: u64, at: DVec3, time: (f64, f64)) -> f32 {
    let (seconds, period) = time;
    let along = seconds / period;
    let index = along.floor();
    let t = (along - index) as f32;
    let ease = t * t * (3.0 - 2.0 * t);
    let key = |index: i64| stream(seed, channel, index).bits();
    let (a, b) = (
        lattice(key(index as i64), at),
        lattice(key(index as i64 + 1), at),
    );
    a + (b - a) * ease
}

/// Returns the broad noise of channel `channel` on the planet of `seed` at the point `at`, in
/// cells of the noise, and at `seconds` of universe time, from 0 to 1. It is smooth in place
/// and in time and never steps.
pub fn broad(seed: u64, channel: u64, at: DVec3, seconds: f64) -> f32 {
    let swell = octave(seed, channel, at, (seconds, PERIODS[0]));
    let ripple = octave(seed, channel + 0x100, at * FINER, (seconds, PERIODS[1]));
    let mean = (swell + RIPPLE * ripple) / (1.0 + RIPPLE);
    (0.5 + CONTRAST * (mean - 0.5)).clamp(0.0, 1.0)
}
