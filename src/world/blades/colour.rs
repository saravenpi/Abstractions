//! The twin of the carpet noise of `turf.wesl`: the share of the second species and the speckle at one spot.
//!
//! A tile bakes both into the colour of each blade, so the blade shader mixes two colours instead of reading
//! nine hashes at every corner. The ground shader still reads the noise itself: both must give one value.

use bevy::prelude::*;

/// Metres after which the turf noise repeats, as `PERIOD` in `turf.wesl`.
const PERIOD: f32 = 256.0;

/// Returns the hash of lattice cell `c`, 0 to 1, as `hash` in `turf.wesl`.
fn hash(c: IVec2) -> f32 {
    let mut h = (c.x as u32).wrapping_mul(73_856_093) ^ (c.y as u32).wrapping_mul(19_349_663);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65535.0
}

/// Returns cell `c` wrapped into `n` cells on both axes, as `wrapped` in `turf.wesl`.
fn wrapped(c: IVec2, n: i32) -> IVec2 {
    IVec2::new(c.x.rem_euclid(n), c.y.rem_euclid(n))
}

/// Returns `a` moved toward `b` by `t`.
fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Returns the smooth step of `x` between `lo` and `hi`, as the shader function of the same name.
fn smooth(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the smooth value noise at `p` over cells of `cell` metres, as `lattice` in `turf.wesl`.
fn lattice(p: Vec2, cell: f32) -> f32 {
    let n = (PERIOD / cell) as i32;
    let q = p / cell;
    let i = q.floor().as_ivec2();
    let f = q - q.floor();
    let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
    let at = |dx: i32, dy: i32| hash(wrapped(i + IVec2::new(dx, dy), n));
    mix(
        mix(at(0, 0), at(1, 0), u.x),
        mix(at(0, 1), at(1, 1), u.x),
        u.y,
    )
}

/// Returns the share of the second species and the speckle of the carpet at wrapped chart position `p`:
/// the `t` and the `speckle` of `carpet` in `turf.wesl`.
pub fn tint(p: Vec2) -> (f32, f32) {
    let clump = lattice(p, 8.0) * 0.65 + lattice(p, 2.0) * 0.35;
    let cells = (p * 4.0).floor().as_ivec2();
    let speckle = hash(wrapped(cells, (PERIOD * 4.0) as i32));
    (smooth(0.35, 0.65, clump), speckle)
}
