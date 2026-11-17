//! The ground by direction: each cube face shows its own stretch of the legacy plane.

use std::f64::consts::FRAC_PI_4;

use bevy::math::{DVec3, Vec2};

use super::{Chart, Face, Globe};
use crate::world::CHUNK_SIZE;
use crate::world::biome::Biome;
use crate::world::surface::Surface;

impl Globe {
    /// Returns the plane position that stands for unit direction `dir`.
    ///
    /// Each face maps to a square of the plane, metric at the face centre, so the ground does
    /// not match across face edges.
    fn plane(&self, dir: DVec3) -> Vec2 {
        let (face, u, v) = Face::locate(dir);
        let half = self.radius() * FRAC_PI_4;
        let shift = 2.0 * self.radius() * (face as u8 as f64 - 2.0);
        Vec2::new((u * half + shift) as f32, (v * half) as f32)
    }

    /// Returns the ground height in metres above sea level in unit direction `dir`.
    pub fn height(&self, dir: DVec3) -> f32 {
        let p = self.plane(dir);
        self.height_flat(p.x, p.y)
    }

    /// Returns the biome in unit direction `dir`, also over water.
    pub fn biome(&self, dir: DVec3) -> Biome {
        let p = self.plane(dir);
        self.biome_flat(p.x, p.y)
    }

    /// Returns what covers the ground in unit direction `dir`.
    pub fn surface(&self, dir: DVec3) -> Surface {
        let p = self.plane(dir);
        self.surface_flat(p.x, p.y)
    }

    /// Returns true when unit direction `dir` lies inside a pond basin.
    pub fn pond_at(&self, dir: DVec3) -> bool {
        let p = self.plane(dir);
        self.pond_flat(p.x, p.y)
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
