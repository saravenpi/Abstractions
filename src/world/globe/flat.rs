//! The legacy plane: the ground as a function of flat world coordinates `(x, z)`.

use bevy::math::Vec2;

use super::Globe;
use crate::world::biome::Biome;
use crate::world::surface::Surface;

impl Globe {
    /// Returns the ground height in metres above sea level at plane position `(x, z)`.
    pub fn height_flat(&self, x: f32, z: f32) -> f32 {
        self.ground(Vec2::new(x, z))
    }

    /// Returns true when plane position `(x, z)` lies inside a pond basin.
    pub fn pond_flat(&self, x: f32, z: f32) -> bool {
        self.in_pond(Vec2::new(x, z))
    }

    /// Returns what covers the ground at plane position `(x, z)`.
    pub fn surface_flat(&self, x: f32, z: f32) -> Surface {
        self.cover(Vec2::new(x, z))
    }

    /// Returns the biome at plane position `(x, z)`, also over water.
    pub fn biome_flat(&self, x: f32, z: f32) -> Biome {
        self.biome_on(Vec2::new(x, z))
    }
}
