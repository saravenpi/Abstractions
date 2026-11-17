//! Gradient noise in space: the sphere path reads this instead of the 3D noise of the `noise` crate.
//!
//! The crate's 3D Perlin costs twice its 2D one, which made a sphere sample three times a flat one.
//! This lattice noise hashes with three multiplies and looks its gradients up without a branch.

/// Gradient directions: the twelve edges of a cube, four of them twice.
const GRADIENTS: [[f64; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [0.0, -1.0, -1.0],
];

/// One large prime per axis, multiplied into the lattice coordinates before they are mixed.
const PRIMES: [u32; 3] = [501_125_321, 1_136_930_381, 1_720_413_743];

/// Gain that gives one octave the same spread of values as the 2D Perlin noise of the plane.
const GAIN: f64 = 1.6;

/// Returns the quintic fade of `t`, flat at 0 and at 1.
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Returns the blend from `a` to `b` by `t`.
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Returns the gradient noise of `seed` at `p`, about -1 to 1, zero on every lattice point.
pub fn gradient(seed: u32, p: [f64; 3]) -> f64 {
    let cell = p.map(f64::floor);
    let d = [p[0] - cell[0], p[1] - cell[1], p[2] - cell[2]];
    let salt = (seed ^ (seed >> 15)).wrapping_mul(0x2C1B_3C6D);
    let key = [0, 1, 2].map(|k| (cell[k] as i64 as u32).wrapping_mul(PRIMES[k]));
    let next = [0, 1, 2].map(|k| key[k].wrapping_add(PRIMES[k]));
    let pick = |k: usize, step: usize| [key[k], next[k]][step];
    let corner = |a: usize, b: usize, c: usize| {
        let h = (salt ^ pick(0, a) ^ pick(1, b) ^ pick(2, c)).wrapping_mul(0x27D4_EB2D);
        let g = GRADIENTS[(h >> 28) as usize];
        g[0] * (d[0] - a as f64) + g[1] * (d[1] - b as f64) + g[2] * (d[2] - c as f64)
    };
    let [u, v, w] = d.map(fade);
    let edge = |b: usize, c: usize| lerp(corner(0, b, c), corner(1, b, c), u);
    let face = |c: usize| lerp(edge(0, c), edge(1, c), v);
    (lerp(face(0), face(1), w) * GAIN).clamp(-1.0, 1.0)
}
