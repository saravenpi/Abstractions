//! Chunk frames: the rigid frame, fixed to the planet, that the geometry of one chunk is stored in.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use crate::world::Terrain;

/// A rigid frame fixed to the planet. `planet = origin + rot * local`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Position of the frame origin in metres from the planet centre.
    pub origin: DVec3,
    /// Rotation from frame axes to planet axes.
    pub rot: DQuat,
}

impl Frame {
    /// The frame of the planet itself.
    pub const PLANET: Frame = Frame {
        origin: DVec3::ZERO,
        rot: DQuat::IDENTITY,
    };

    /// Returns the tangent frame at chart position `at` of `terrain`: the origin on the sea level
    /// sphere, +Y up, +X along chart x and +Z completing them, close to chart z.
    pub fn at(terrain: &Terrain, at: Vec2) -> Self {
        let up = terrain.up(at.x, at.y);
        let east = terrain.chart().east(at.x, at.y);
        Self {
            origin: terrain.embed(at.x, 0.0, at.y),
            rot: DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up))),
        }
    }

    /// Returns the frame coordinates of a planet position.
    pub fn local(&self, pos: DVec3) -> Vec3 {
        (self.rot.inverse() * (pos - self.origin)).as_vec3()
    }

    /// Returns a planet direction in frame axes.
    pub fn turn(&self, dir: DVec3) -> Vec3 {
        (self.rot.inverse() * dir).as_vec3()
    }
}
