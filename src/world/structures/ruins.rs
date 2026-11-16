//! Ruins: the standing stone circle, the broken round tower and the obelisk.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::dress::on_ground;
use super::palette::{RUNE, STONE, STONE_DARK, STONE_LIGHT};

/// Distance from the tower center to the middle of its walls.
const TOWER_R: f32 = 4.2;

/// Wall thickness of the tower.
const TOWER_T: f32 = 0.9;

/// Number of wall segments of the tower.
const TOWER_SIDES: u32 = 8;

/// Adds one stone standing on the ground at the current frame, capped with a small gable.
fn standing_stone(b: &mut Build, seed: u32, height: f32) {
    b.settle(0.0, 0.0);
    let h = height * (0.7 + b.roll(seed) * 0.5);
    b.cuboid(
        Vec3::new(-0.55, -1.2, -0.35),
        Vec3::new(0.55, h, 0.35),
        Mat::solid(STONE),
    );
    b.gable(
        Vec3::new(-0.55, h, -0.35),
        Vec3::new(0.55, h, 0.35),
        0.45,
        Mat::deco(STONE_LIGHT),
    );
}

/// Draws a dais with an altar inside a ring of standing stones.
pub fn stone_circle(b: &mut Build) {
    b.round_foundation(2.0, 8, 0.25, Mat::solid(STONE_DARK));
    b.cuboid(
        Vec3::new(-0.7, 0.25, -0.5),
        Vec3::new(0.7, 1.0, 0.5),
        Mat::solid(STONE),
    );
    b.cuboid(
        Vec3::new(-0.85, 1.0, -0.65),
        Vec3::new(0.85, 1.15, 0.65),
        Mat::solid(STONE_LIGHT),
    );
    for i in 0..8 {
        let a = i as f32 / 8.0 * TAU;
        b.frame(Vec2::new(a.sin(), a.cos()) * 7.0, a);
        standing_stone(b, i, if i % 4 == 3 { 1.2 } else { 3.0 });
    }
}

/// Draws wall segment `i` of the tower in the current frame. Segment 0 holds the doorway.
fn tower_wall(b: &mut Build, i: u32) {
    let half = TOWER_R * (PI / TOWER_SIDES as f32).tan() + 0.2;
    let t = TOWER_T * 0.5;
    let mat = Mat::solid(if i.is_multiple_of(2) {
        STONE
    } else {
        STONE_DARK
    });
    if i == 0 {
        b.cuboid(
            Vec3::new(-half, 0.0, -t),
            Vec3::new(-half + 0.9, 5.0, t),
            mat,
        );
        b.cuboid(Vec3::new(half - 0.9, 0.0, -t), Vec3::new(half, 5.0, t), mat);
        b.cuboid(
            Vec3::new(-half + 0.9, 2.4, -t),
            Vec3::new(half - 0.9, 3.2, t),
            mat,
        );
        return;
    }
    let h = 2.5 + b.roll(i) * 6.0;
    b.cuboid(Vec3::new(-half, 0.0, -t), Vec3::new(half, h, t), mat);
}

/// Draws a broken round tower with an open doorway and a walkable floor.
pub fn tower(b: &mut Build) {
    b.round_foundation(5.0, 12, 0.0, Mat::solid(STONE_DARK));
    for i in 0..TOWER_SIDES {
        let a = i as f32 / TOWER_SIDES as f32 * TAU;
        b.frame(Vec2::new(a.sin(), a.cos()) * TOWER_R, a);
        tower_wall(b, i);
    }
    b.begin(b.site);
    b.ramp(Vec3::new(0.0, 0.0, 4.9), 2.0);
    for x in [-1.6, 1.6] {
        let base = on_ground(b, x, 5.6);
        b.torch_post(base);
    }
    b.rock(3.2, 6.0, 0.6, 1);
    b.rock(-3.8, 5.6, 0.45, 2);
    b.rock(5.5, -1.0, 0.5, 3);
}

/// Draws a carved monolith on a two-step base with four small stones around it.
pub fn obelisk(b: &mut Build) {
    let depth = b.site.depth;
    b.cuboid(
        Vec3::new(-2.2, -depth, -2.2),
        Vec3::new(2.2, 0.3, 2.2),
        Mat::solid(STONE_DARK),
    );
    b.cuboid(
        Vec3::new(-1.5, 0.3, -1.5),
        Vec3::new(1.5, 0.6, 1.5),
        Mat::solid(STONE),
    );
    b.pillar(Vec3::Y * 0.6, Vec2::new(0.95, 8.0), 4, Mat::solid(STONE));
    b.pillar(Vec3::Y * 4.5, Vec2::new(1.0, 0.35), 4, Mat::deco(RUNE));
    b.spire(Vec3::Y * 8.6, Vec2::new(0.95, 1.6), 4, STONE_LIGHT);
    for i in 0..4 {
        let a = (i as f32 + 0.5) / 4.0 * TAU;
        b.frame(Vec2::new(a.sin(), a.cos()) * 3.6, a);
        standing_stone(b, i, 1.8);
    }
}
