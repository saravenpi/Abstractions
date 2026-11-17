//! Embedding: carries chunk geometry built in chart coordinates into the frame of the chunk on
//! the globe.

use bevy::prelude::*;

use crate::world::Terrain;
use crate::world::mesh::Frame;
use crate::world::scatter::chunk_center;

#[cfg(test)]
mod legacy;

/// The one map from chunk space, where `(x, z)` are chart offsets from the chunk centre and `y`
/// is the height above sea level, to the tangent frame at the middle of the chunk.
///
/// Every mesh, collider and child transform a generator builds in chunk space goes through it.
pub struct Embedding {
    terrain: Terrain,
    center: Vec2,
    frame: Frame,
}

impl Embedding {
    /// Returns the embedding of chunk `coord` of the chart of `terrain`.
    pub fn of(terrain: &Terrain, coord: IVec2) -> Self {
        let center = chunk_center(coord);
        Self {
            terrain: terrain.clone(),
            center,
            frame: Frame::at(terrain, center),
        }
    }

    /// Returns the frame of the chunk: the tangent frame at its middle, on the sea level sphere.
    pub fn frame(&self) -> Frame {
        self.frame
    }

    /// Returns the chart position of the middle of the chunk.
    pub fn center(&self) -> Vec2 {
        self.center
    }

    /// Returns the chunk frame position of chunk space point `p`.
    pub fn point(&self, p: Vec3) -> Vec3 {
        let at = self.center + Vec2::new(p.x, p.z);
        self.frame.local(self.terrain.embed(at.x, p.y, at.y))
    }

    /// Returns the rotation that carries directions at chunk space point `p` into the chunk
    /// frame: local up stays up and chart x stays chart x.
    pub fn turn(&self, p: Vec3) -> Quat {
        let here = Frame::at(&self.terrain, self.center + Vec2::new(p.x, p.z));
        (self.frame.rot.inverse() * here.rot).normalize().as_quat()
    }
}
