//! Wooden cabins and the village of three cabins around a well.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{DARK_WOOD, PLANK, ROOF, SNOW, STONE, STONE_DARK, WATER, WOOD};
use crate::world::Biome;

/// Half length of a cabin along x.
const HX: f32 = 3.0;

/// Half depth of a cabin along z.
const HZ: f32 = 2.4;

/// Wall thickness.
const T: f32 = 0.3;

/// Wall height.
const H: f32 = 2.5;

/// Wall boxes of a cabin: back, left, right, two door sides and the lintel.
const WALLS: [([f32; 3], [f32; 3]); 6] = [
    ([-HX, 0.0, -HZ], [HX, H, -HZ + T]),
    ([-HX, 0.0, -HZ + T], [-HX + T, H, HZ]),
    ([HX - T, 0.0, -HZ + T], [HX, H, HZ]),
    ([-HX + T, 0.0, HZ - T], [-0.8, H, HZ]),
    ([0.8, 0.0, HZ - T], [HX - T, H, HZ]),
    ([-0.8, 2.0, HZ - T], [0.8, H, HZ]),
];

/// Draws one cabin in the current frame, door toward +z. `small` drops chimney, posts and furniture.
pub fn cabin_body(b: &mut Build, small: bool) {
    b.foundation(Vec2::new(HX + 0.3, HZ + 0.3), Mat::solid(STONE_DARK));
    b.boxes(&WALLS, Mat::solid(WOOD));
    let (lo, hi) = (
        Vec3::new(-HX - 0.5, H, -HZ - 0.5),
        Vec3::new(HX + 0.5, H, HZ + 0.5),
    );
    b.gable(lo, hi, 1.8, Mat::deco(ROOF));
    if b.terrain.biome(b.pos.x, b.pos.y) == Biome::Taiga {
        b.gable(
            lo + Vec3::Y * 0.12,
            hi + Vec3::Y * 0.12,
            1.8,
            Mat::deco(SNOW),
        );
    }
    b.ramp(Vec3::new(0.0, 0.0, HZ + 0.3), 1.6);
    if !small {
        furnish(b);
    }
}

/// Adds the chimney, corner posts, a table and a bed to a cabin.
fn furnish(b: &mut Build) {
    b.cuboid(
        Vec3::new(HX, 0.0, -0.5),
        Vec3::new(HX + 0.8, H + 2.2, 0.5),
        Mat::solid(STONE),
    );
    for (x, z) in [(-HX, -HZ), (HX, -HZ), (-HX, HZ), (HX, HZ)] {
        let (lo, hi) = (
            Vec3::new(x - 0.18, 0.0, z - 0.18),
            Vec3::new(x + 0.18, H + 0.1, z + 0.18),
        );
        b.cuboid(lo, hi, Mat::deco(DARK_WOOD));
    }
    b.cuboid(
        Vec3::new(-1.8, 0.0, -1.7),
        Vec3::new(-0.6, 0.8, -0.9),
        Mat::solid(PLANK),
    );
    b.cuboid(
        Vec3::new(1.2, 0.0, -1.9),
        Vec3::new(HX - T, 0.6, 0.2),
        Mat::solid(PLANK),
    );
}

/// Draws a single wooden cabin.
pub fn cabin(b: &mut Build) {
    cabin_body(b, false);
}

/// Draws a stone well, with three small cabins turned toward it.
pub fn village(b: &mut Build) {
    b.round_foundation(1.1, 8, 0.8, Mat::solid(STONE));
    b.spire(Vec3::Y * 0.85, Vec2::new(0.7, 0.001), 6, WATER);
    for i in 0..3 {
        let a = i as f32 / 3.0 * TAU + 0.5;
        b.frame(Vec2::new(a.sin(), a.cos()) * 10.0, a + PI);
        b.settle(4.0, 0.35);
        cabin_body(b, true);
    }
}
