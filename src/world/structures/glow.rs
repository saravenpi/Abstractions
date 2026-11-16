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
    /// Fire in the hearth of a house.
    Hearth,
    /// Candle or small lantern inside a house.
    Candle,
}

impl LightKind {
    /// Every kind, in declaration order, so `kind as usize` indexes per-kind tables.
    pub const ALL: [LightKind; 6] = [
        LightKind::Lamp,
        LightKind::Torch,
        LightKind::Campfire,
        LightKind::Brazier,
        LightKind::Hearth,
        LightKind::Candle,
    ];

    /// Returns the warm color of the light a kind gives: open fires are deep orange, the lamp and candle paler.
    pub fn tint(self) -> Color {
        let [r, g, b] = match self {
            LightKind::Lamp => [1.0, 0.76, 0.48],
            LightKind::Torch => [1.0, 0.62, 0.3],
            LightKind::Campfire => [1.0, 0.56, 0.24],
            LightKind::Brazier => [1.0, 0.58, 0.26],
            LightKind::Hearth => [1.0, 0.6, 0.3],
            LightKind::Candle => [1.0, 0.74, 0.44],
        };
        Color::srgb(r, g, b)
    }
}

/// How far a wall torch holds its flame from the wall, in meters: far enough not to burn a white spot on it.
pub const WALL_REACH: f32 = 0.4;

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

    /// Adds a torch on an arm from a wall facing local +z, with the wall surface at the local point `at`.
    pub fn wall_torch(&mut self, at: Vec3) {
        let arm = (
            Vec3::new(-0.03, -0.06, 0.0),
            Vec3::new(0.03, 0.0, WALL_REACH),
        );
        let grip = (
            Vec3::new(-0.04, -0.25, WALL_REACH - 0.04),
            Vec3::new(0.04, 0.3, WALL_REACH + 0.04),
        );
        for (lo, hi) in [arm, grip] {
            self.cuboid(at + lo, at + hi, Mat::deco(DARK_WOOD));
        }
        self.glow(LightKind::Torch, at + Vec3::new(0.0, 0.45, WALL_REACH));
    }
}
