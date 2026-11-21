//! The noises the plant cover is read from on one planet, so that single places can be read without a chunk.

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::world::Terrain;

/// Frequencies per metre of the noise of barren stretches, of clearings and of the stands of each species.
const FREQ: [f64; 3] = [1.0 / 380.0, 1.0 / 70.0, 1.0 / 45.0];

/// Salts of the planet seed the three noises are drawn from.
const SALTS: [u32; 3] = [0xC0E5, 0x61AD, 0x57A0];

/// Cover level a lush planet is cut at, and how much higher the patchiest planet cuts it.
const CUT: (f32, f32) = (0.12, 0.6);

/// Half the width of the easing between barren and covered ground, in cover level.
const SOFT: f32 = 0.1;

/// Returns the share `x` has moved from `a` to `b`, eased and clamped to 0 to 1.
fn ease(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The noises the cover is read from on one planet, so that single places can be read without a chunk.
pub struct Sampler {
    /// The noises of barren stretches, of clearings and of the stands of each species.
    fields: [Perlin; 3],
    /// Cover level the planet is cut at.
    cut: f32,
}

impl Sampler {
    /// Returns the sampler of `terrain` on a planet of `patchiness`.
    pub fn new(terrain: &Terrain, patchiness: f32) -> Self {
        Self {
            fields: SALTS.map(|salt| Perlin::new(terrain.seed() as u32 ^ salt)),
            cut: CUT.0 + CUT.1 * patchiness,
        }
    }

    /// Returns the cover, kin key and moisture `wet` at chart position `w` of `terrain`.
    pub fn at(&self, terrain: &Terrain, w: Vec2, wet: f32) -> [f32; 3] {
        let p = terrain.embed(w.x, 0.0, w.y);
        let read = |f: usize| self.fields[f].get((p * FREQ[f]).to_array()) as f32;
        let level = 0.5 + 0.8 * read(0) + 0.35 * read(1) + 0.3 * (wet - 0.5);
        let key = (0.5 + 1.3 * read(2)).clamp(0.0, 1.0);
        [ease(self.cut - SOFT, self.cut + SOFT, level), key, wet]
    }
}
