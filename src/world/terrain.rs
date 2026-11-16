//! The terrain height field: the single source of truth for where land is.

use bevy::prelude::*;

use super::SEA_LEVEL;
use super::biome::{Biome, Climate};
use super::islands::Shape;
use super::surface::{Surface, classify};

mod ponds;
mod spawn;

/// Finite difference step used to measure slopes.
const SLOPE_STEP: f32 = 1.5;

/// Deterministic infinite terrain built from a seed.
#[derive(Resource, Clone)]
pub struct Terrain {
    seed: u64,
    shape: Shape,
    climate: Climate,
}

impl Terrain {
    /// Builds the terrain for a seed. The same seed always gives the same world.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            shape: Shape::new(seed),
            climate: Climate::new(seed),
        }
    }

    /// Returns the seed this terrain was built from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the ground height at world position `(x, z)`.
    pub fn height(&self, x: f32, z: f32) -> f32 {
        self.carve(x, z, self.shape.elevation(x, z, &self.climate))
    }

    /// Returns what covers the ground at world position `(x, z)`.
    pub fn surface(&self, x: f32, z: f32) -> Surface {
        let h = self.height(x, z);
        if h <= SEA_LEVEL && self.in_pond(x, z) {
            return Surface::Pond;
        }
        let e = SLOPE_STEP;
        let dx = (self.height(x + e, z) - self.height(x - e, z)) / (2.0 * e);
        let dz = (self.height(x, z + e) - self.height(x, z - e)) / (2.0 * e);
        classify(h, dx.hypot(dz), self.biome(x, z))
    }

    /// Returns the biome at world position `(x, z)`, also over water.
    pub fn biome(&self, x: f32, z: f32) -> Biome {
        self.climate.biome(x, z, self.shape.base(x, z))
    }
}

#[cfg(test)]
mod tests;
