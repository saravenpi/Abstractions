//! The legacy plane: the ground as a function of flat world coordinates `(x, z)`.

use super::Globe;
use crate::world::SEA_LEVEL;
use crate::world::biome::Biome;
use crate::world::surface::{Surface, classify};

/// Finite difference step used to measure slopes.
const SLOPE_STEP: f32 = 1.5;

impl Globe {
    /// Returns the ground height in metres above sea level at plane position `(x, z)`.
    pub fn height_flat(&self, x: f32, z: f32) -> f32 {
        self.carve(x, z, self.shape.elevation(x, z, &self.climate))
    }

    /// Returns true when plane position `(x, z)` lies inside a pond basin.
    pub fn pond_flat(&self, x: f32, z: f32) -> bool {
        self.in_pond(x, z)
    }

    /// Returns what covers the ground at plane position `(x, z)`.
    pub fn surface_flat(&self, x: f32, z: f32) -> Surface {
        let h = self.height_flat(x, z);
        if h <= SEA_LEVEL && self.in_pond(x, z) {
            return Surface::Pond;
        }
        let e = SLOPE_STEP;
        let dx = (self.height_flat(x + e, z) - self.height_flat(x - e, z)) / (2.0 * e);
        let dz = (self.height_flat(x, z + e) - self.height_flat(x, z - e)) / (2.0 * e);
        classify(h, dx.hypot(dz), self.biome_flat(x, z))
    }

    /// Returns the biome at plane position `(x, z)`, also over water.
    pub fn biome_flat(&self, x: f32, z: f32) -> Biome {
        self.climate.biome(x, z, self.shape.base(x, z))
    }
}
