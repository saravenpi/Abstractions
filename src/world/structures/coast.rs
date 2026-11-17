//! The striped lighthouse on the shore.

use std::f32::consts::{FRAC_PI_4, FRAC_PI_8};

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{DARK_WOOD, GLASS, RED, STONE_DARK, WHITE};

/// Radius of the lowest drum of the tower, out to its eight corners.
pub const BASE: f32 = 2.8;

/// Draws a tall striped tower with a gallery, a glowing lantern and a red cap. The drums do not turn with
/// the site, so the door is turned to lie flat on the face of the tower nearest to the front of the site.
pub fn lighthouse(b: &mut Build) {
    b.round_foundation(3.6, 10, 0.0, Mat::solid(STONE_DARK));
    for i in 0..6 {
        let rgb = if i % 2 == 0 { RED } else { WHITE };
        let size = Vec2::new(BASE - 0.2 * i as f32, 2.3);
        b.pillar(Vec3::Y * (i as f32 * 2.3), size, 8, Mat::solid(rgb));
    }
    b.pillar(
        Vec3::Y * 13.8,
        Vec2::new(2.5, 0.3),
        8,
        Mat::solid(STONE_DARK),
    );
    b.spire(Vec3::Y * 14.1, Vec2::new(2.5, 0.001), 8, STONE_DARK);
    b.pillar(Vec3::Y * 14.1, Vec2::new(1.1, 1.6), 8, Mat::deco(GLASS));
    b.spire(Vec3::Y * 15.7, Vec2::new(1.6, 1.4), 8, RED);
    let face = BASE * FRAC_PI_8.cos();
    let toward = ((b.site.yaw / FRAC_PI_4).floor() + 0.5) * FRAC_PI_4;
    b.frame(Vec2::ZERO, toward - b.site.yaw);
    b.cuboid(
        Vec3::new(-0.45, 0.0, face - 0.1),
        Vec3::new(0.45, 1.9, face + 0.1),
        Mat::deco(DARK_WOOD),
    );
    b.frame(Vec2::ZERO, 0.0);
}
