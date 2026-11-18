//! Land shape: warped continent mask with bays, islets, sandbars, cliffs and the elevation curves.

mod elevation;
mod field;
mod fractal;
mod point;
mod point2;
mod point3;
mod relief;
mod seabed;
mod solid;

#[cfg(test)]
mod tests;

use noise::{Fbm, MultiFractal, Perlin};

pub use point::{Point, plus};
pub(in crate::world) use solid::gradient;

use relief::Relief;
use seabed::Seabed;

/// Continent noise value above which the ground rises out of the sea.
const LAND_LEVEL: f32 = 0.16;

/// Sea-side inland measure below which the cliff weight no longer changes the sea floor.
const CLIFF_REACH: f32 = -0.38;

/// Where the large warp is read for each axis of the displacement, in noise units.
const WARP_SHIFT: [[f64; 3]; 3] = [[0.0; 3], [31.7, -17.3, 12.4], [-23.9, 8.6, -41.2]];

/// Where the small coast warp is read for each axis of the displacement, in noise units.
const COAST_SHIFT: [[f64; 3]; 3] = [[0.0; 3], [9.1, -4.3, 6.7], [-5.2, 2.8, -7.9]];

/// Returns 0 below `a`, 1 above `b` and a smooth blend between (also works when `a > b`).
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Derives the noise seed number `k` from the world seed.
pub fn sub_seed(seed: u64, k: u32) -> u32 {
    let h = (seed ^ (seed >> 32)) as u32;
    h.wrapping_add(k.wrapping_mul(0x9E37_79B1)) ^ k.wrapping_mul(0x85EB_CA6B)
}

/// The noise fields that shape the land and the sea floor.
#[derive(Clone)]
pub struct Shape {
    warp: Perlin,
    coast: Perlin,
    land: Fbm<Perlin>,
    bridge: Perlin,
    stack: Perlin,
    cliff: Perlin,
    relief: Relief,
    seabed: Seabed,
    /// Factor on the height of the mountains on a sphere, from the pull of the planet.
    gain: f32,
    /// Factor on the frequency of the continents on a sphere, from the size of the planet.
    continent: f64,
}

impl Shape {
    /// Builds the noise fields from the world seed.
    pub fn new(seed: u64) -> Self {
        Self {
            warp: Perlin::new(sub_seed(seed, 1)),
            coast: Perlin::new(sub_seed(seed, 8)),
            land: Fbm::<Perlin>::new(sub_seed(seed, 2))
                .set_octaves(4)
                .set_frequency(1.0 / 640.0),
            bridge: Perlin::new(sub_seed(seed, 9)),
            stack: Perlin::new(sub_seed(seed, 10)),
            cliff: Perlin::new(sub_seed(seed, 11)),
            relief: Relief::new(seed),
            seabed: Seabed::new(seed),
            gain: 1.0,
            continent: 1.0,
        }
    }

    /// Returns the fields with the mountains `gain` times as high and the continents read at `continent`
    /// times their frequency, both on a sphere only: the legacy plane keeps the fields as they are.
    pub fn scaled(self, gain: f32, continent: f64) -> Self {
        Self {
            gain,
            continent,
            ..self
        }
    }

    /// Returns the continent noise at `p`, read at the frequency of the planet on a sphere.
    fn land<P: Point>(&self, p: P) -> f32 {
        match P::SOLID {
            true => p.mul(self.continent).get(&self.land) as f32,
            false => p.get(&self.land) as f32,
        }
    }

    /// Returns the unwarped continent measure at `p`: above 0 on big land, below 0 at sea.
    pub fn base<P: Point>(&self, p: P) -> f32 {
        self.land(p) - LAND_LEVEL
    }
}
