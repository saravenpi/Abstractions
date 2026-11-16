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

    /// Returns the height the finest terrain mesh draws at `(x, z)`: its 2 m triangles, interpolated.
    ///
    /// Anything that must touch the visible ground stands on this, not on [`Terrain::height`].
    pub fn mesh_height(&self, x: f32, z: f32) -> f32 {
        let step = super::mesh::GRID_STEP;
        let (i, j) = ((x / step).floor(), (z / step).floor());
        let (u, v) = (x / step - i, z / step - j);
        let h = |a: f32, b: f32| self.height((i + a) * step, (j + b) * step);
        if u + v <= 1.0 {
            h(0.0, 0.0) + u * (h(1.0, 0.0) - h(0.0, 0.0)) + v * (h(0.0, 1.0) - h(0.0, 0.0))
        } else {
            h(1.0, 1.0)
                + (1.0 - u) * (h(0.0, 1.0) - h(1.0, 1.0))
                + (1.0 - v) * (h(1.0, 0.0) - h(1.0, 1.0))
        }
    }

    /// Returns true when `(x, z)` lies inside a pond basin.
    pub fn pond_at(&self, x: f32, z: f32) -> bool {
        self.in_pond(x, z)
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
