//! The ground by direction: every field is read at the 3D point `dir * radius`, so nothing shows a seam.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

use super::{Chart, Face, Globe};
use crate::world::CHUNK_SIZE;
use crate::world::biome::Biome;
use crate::world::surface::Surface;

impl Globe {
    /// Returns the point of the sea level sphere, in metres from the centre, in unit direction `dir`.
    fn at(&self, dir: DVec3) -> DVec3 {
        dir * self.radius()
    }

    /// Returns the ground height in metres above sea level in unit direction `dir`.
    pub fn height(&self, dir: DVec3) -> f32 {
        self.ground(self.at(dir))
    }

    /// Returns the biome in unit direction `dir`, also over water.
    pub fn biome(&self, dir: DVec3) -> Biome {
        self.biome_on(self.at(dir))
    }

    /// Returns what covers the ground in unit direction `dir`, the slope being measured along two
    /// tangent directions.
    pub fn surface(&self, dir: DVec3) -> Surface {
        self.cover(self.at(dir))
    }

    /// Returns true when unit direction `dir` lies inside a pond basin.
    pub fn pond_at(&self, dir: DVec3) -> bool {
        self.in_pond(self.at(dir))
    }

    /// Returns the quadtree level whose nodes are closest to `CHUNK_SIZE` metres wide.
    pub fn leaf_level(&self) -> u8 {
        let nodes = 2.0 * self.radius() * FRAC_PI_4 / f64::from(CHUNK_SIZE);
        nodes.log2().round().clamp(0.0, 20.0) as u8
    }

    /// Returns the chart of `face` in which a node of [`Globe::leaf_level`] is `CHUNK_SIZE` wide.
    pub fn face_chart(&self, face: Face) -> Chart {
        let scale = f64::from(CHUNK_SIZE) * (1u64 << self.leaf_level()) as f64 * 0.5;
        Chart::Face { face, scale }
    }
}
