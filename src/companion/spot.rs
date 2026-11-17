//! Where the companion wants to be: beside the player, clear of the ground and the water.

use bevy::prelude::*;

use super::tune::{BEHIND, REACH, RISE, SIDE};
use crate::space::{Anchor, altitude, up};
use crate::world::{SEA_LEVEL, Terrain};

/// Where the player's eyes are and which way they look, in render space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stance {
    /// Position of the eyes.
    pub eye: Vec3,
    /// Unit view direction.
    pub forward: Vec3,
    /// Unit local vertical.
    pub up: Vec3,
}

impl Stance {
    /// Returns the view direction flattened onto the horizontal plane, even when looking straight up or down.
    pub fn level(&self) -> Vec3 {
        (self.forward - self.up * self.forward.dot(self.up))
            .try_normalize()
            .unwrap_or_else(|| self.up.any_orthonormal_vector())
    }

    /// Returns the horizontal direction to the right of the view.
    pub fn right(&self) -> Vec3 {
        self.level().cross(self.up)
    }

    /// Returns the comfortable spot in the horizontal direction `side` of the eyes, the place
    /// the companion chose, before any ground clearance.
    pub fn rest(&self, side: Vec3) -> Vec3 {
        self.eye + side * REACH + self.up * RISE
    }

    /// Returns the spot just behind the player where a companion left far behind reappears.
    pub fn behind(&self) -> Vec3 {
        self.eye - self.level() * BEHIND + self.right() * SIDE * 0.5
    }
}

/// Returns `pos` raised along `up` until it is `clearance` above a floor that is `gap` below it.
pub fn lift(pos: Vec3, up: Vec3, gap: f32, clearance: f32) -> Vec3 {
    pos + up * (clearance - gap).max(0.0)
}

/// Reads the ground and the water under a render position.
#[derive(Clone, Copy)]
pub struct Probe<'a> {
    /// The render anchor.
    pub anchor: &'a Anchor,
    /// The terrain of the anchored planet.
    pub terrain: &'a Terrain,
}

impl Probe<'_> {
    /// Returns the local vertical at `pos`.
    pub fn up(&self, pos: Vec3) -> Vec3 {
        up(self.anchor, pos)
    }

    /// Returns the height in meters of `pos` above the ground or the water surface, whichever is higher.
    pub fn gap(&self, pos: Vec3) -> f32 {
        let at = self.terrain.locate(self.anchor.to_frame(pos));
        let floor = self.terrain.height(at.x, at.z).max(SEA_LEVEL);
        altitude(self.anchor, self.terrain.globe(), pos) - floor
    }

    /// Returns `pos` raised along the local vertical until it is `clearance` above the ground and the water.
    pub fn clear(&self, pos: Vec3, clearance: f32) -> Vec3 {
        lift(pos, self.up(pos), self.gap(pos), clearance)
    }
}
