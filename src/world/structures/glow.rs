//! Light points and torches: what a structure asks the world to light, drawn as torch posts and wall torches.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{DARK_WOOD, WOOD};

/// What kind of light a point is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    /// Steady street lamp.
    Lamp,
    /// Flickering torch.
    Torch,
    /// Campfire in a ring of stones.
    Campfire,
    /// Brazier on a stand.
    Brazier,
}

/// One light of a structure, in chunk space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightPoint {
    /// Position of the flame or lamp.
    pub pos: Vec3,
    /// Which kind of light.
    pub kind: LightKind,
}

impl Build<'_> {
    /// Records a light of `kind` at the local point `at`.
    pub fn glow(&mut self, kind: LightKind, at: Vec3) {
        let pos = self.pt(at);
        self.lights.push(LightPoint { pos, kind });
    }

    /// Adds a standing torch on a wooden post whose foot is at the local point `base`.
    pub fn torch_post(&mut self, base: Vec3) {
        self.pillar(
            base - Vec3::Y * 0.1,
            Vec2::new(0.05, 1.6),
            4,
            Mat::deco(DARK_WOOD),
        );
        let (lo, hi) = (Vec3::new(-0.08, 1.5, -0.08), Vec3::new(0.08, 1.62, 0.08));
        self.cuboid(base + lo, base + hi, Mat::deco(WOOD));
        self.glow(LightKind::Torch, base + Vec3::Y * 1.78);
    }

    /// Adds a torch bracket on a wall facing local +z, with the wall surface at the local point `at`.
    pub fn wall_torch(&mut self, at: Vec3) {
        let (lo, hi) = (Vec3::new(-0.04, -0.25, 0.0), Vec3::new(0.04, 0.3, 0.1));
        self.cuboid(at + lo, at + hi, Mat::deco(DARK_WOOD));
        self.glow(LightKind::Torch, at + Vec3::new(0.0, 0.45, 0.1));
    }
}
