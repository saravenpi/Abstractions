//! Small props shared by camps and villages: crates, barrels, hay, seats and fires.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::glow::LightKind;
use super::palette::{DARK_WOOD, PLANK, STONE_DARK, THATCH, WOOD};

/// Returns the point of the local column `(x, z)` that lies on the ground.
pub fn on_ground(b: &Build, x: f32, z: f32) -> Vec3 {
    Vec3::new(x, b.ground_at(x, z) - b.base_y, z)
}

/// Adds a wooden crate of side `size` standing on `base`.
pub fn crate_at(b: &mut Build, base: Vec3, size: f32) {
    let h = Vec3::new(size * 0.5, 0.0, size * 0.5);
    b.cuboid(
        base - h - Vec3::Y * 0.1,
        base + h + Vec3::Y * size,
        Mat::solid(PLANK),
    );
}

/// Adds an upright barrel standing on `base`.
pub fn barrel(b: &mut Build, base: Vec3) {
    b.pillar(
        base - Vec3::Y * 0.1,
        Vec2::new(0.36, 1.0),
        6,
        Mat::solid(DARK_WOOD),
    );
}

/// Adds a hay bale standing on `base`.
pub fn hay(b: &mut Build, base: Vec3) {
    let h = Vec3::new(0.55, 0.0, 0.4);
    b.cuboid(
        base - h - Vec3::Y * 0.1,
        base + h + Vec3::Y * 0.7,
        Mat::solid(THATCH),
    );
}

/// Adds a log seat of length `len` along local x, standing on `base`.
pub fn seat(b: &mut Build, base: Vec3, len: f32) {
    let h = Vec3::new(len * 0.5, 0.0, 0.22);
    b.cuboid(
        base - h - Vec3::Y * 0.1,
        base + h + Vec3::Y * 0.42,
        Mat::solid(WOOD),
    );
}

/// Adds a ring of stones on `base`, with flames unless the fire is `cold`.
pub fn fire(b: &mut Build, base: Vec3, cold: bool) {
    for i in 0..6 {
        let a = i as f32 / 6.0 * TAU;
        let at = base + Vec3::new(a.sin() * 0.8, -0.1, a.cos() * 0.8);
        b.pillar(at, Vec2::new(0.24, 0.38), 5, Mat::deco(STONE_DARK));
    }
    if cold {
        let ash = base - Vec3::Y * 0.1;
        b.spire(ash, Vec2::new(0.5, 0.2), 5, [0.18; 3]);
    } else {
        b.glow(LightKind::Campfire, base + Vec3::Y * 0.5);
    }
}
