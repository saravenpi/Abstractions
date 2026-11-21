//! The ground under render positions: altitude, local axes and terrain, read through the anchor.

use bevy::prelude::*;

use super::eye::Eye;
use crate::space::{Anchor, altitude, up};
use crate::world::Terrain;

/// The anchored planet as the effects read it: every position is a render position.
///
/// Heights are meters above sea level along the local vertical, never render `y`. Terrain is
/// read through whatever chart the shared terrain view has, so a change of cube face changes
/// nothing here.
#[derive(Clone, Copy)]
pub struct Ground<'a> {
    /// The active frame anchor, mapping between frame coordinates and render space.
    pub anchor: &'a Anchor,
    terrain: &'a Terrain,
    /// The camera this frame. Its local frame, +Y up, is the one headings and offsets use.
    pub eye: &'a Eye,
}

impl<'a> Ground<'a> {
    /// Returns the ground read through `anchor` and `terrain` around `eye`.
    pub fn new(anchor: &'a Anchor, terrain: &'a Terrain, eye: &'a Eye) -> Self {
        Self {
            anchor,
            terrain,
            eye,
        }
    }

    /// Returns the chart position `(x, z)` of render position `pos` in the terrain view, and
    /// its height above sea level in `y`.
    pub fn chart(&self, pos: Vec3) -> Vec3 {
        self.terrain.locate(self.anchor.to_frame(pos))
    }

    /// Returns the height of render position `pos` above sea level.
    pub fn altitude(&self, pos: Vec3) -> f32 {
        altitude(self.anchor, self.terrain.globe(), pos)
    }

    /// Returns the height above sea level of the ground under render position `pos`. This
    /// samples the terrain.
    pub fn height(&self, pos: Vec3) -> f32 {
        let at = self.chart(pos);
        self.terrain.height(at.x, at.z)
    }

    /// Returns the height of the liquid surface of the planet, which is below any ground where the planet
    /// has no sea: the floor under whatever floats or flies.
    pub fn sea_level(&self) -> f32 {
        self.terrain.globe().sea_level()
    }

    /// Returns `pos` moved along its vertical to `level` meters above sea level.
    pub fn raised(&self, pos: Vec3, level: f32) -> Vec3 {
        pos + up(self.anchor, pos) * (level - self.altitude(pos))
    }

    /// Returns the vector from `from` to `to` in the local frame of the eye: `y` is up.
    pub fn span(&self, from: Vec3, to: Vec3) -> Vec3 {
        self.eye.tilt.inverse() * (to - from)
    }

    /// Returns a vector of the local frame of the eye, where `y` is up, in render axes.
    pub fn turn(&self, local: Vec3) -> Vec3 {
        self.eye.tilt * local
    }
}
