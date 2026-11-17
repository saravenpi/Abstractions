//! The wooden cabin.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::doors::Doorway;
use super::eaves::Cap;
use super::palette::{DARK_WOOD, PLANK, ROOF, STONE, STONE_DARK, WOOD};
use super::tags::Tag;
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

/// Draws the cabin in the current frame, door toward +z.
pub fn cabin(b: &mut Build) {
    b.foundation(Vec2::new(HX + 0.3, HZ + 0.3), Mat::solid(STONE_DARK));
    b.tagged(Tag::Wall, |b| b.boxes(&WALLS, Mat::solid(WOOD)));
    let cap = Cap {
        roof: ROOF,
        wall: WOOD,
        hip: false,
        snow: b.terrain.biome(b.pos.x, b.pos.y) == Biome::Taiga,
    };
    let walls = (Vec3::new(-HX, H, -HZ), Vec3::new(HX, H, HZ));
    b.pitched_over(walls, 0.5, 1.8, &cap);
    let door = Doorway {
        x: (-0.8, 0.8),
        top: 2.0,
        z: (HZ - T, HZ),
    };
    b.door_frame(&door, DARK_WOOD);
    b.ramp(Vec3::new(0.0, 0.0, HZ + 0.3), 1.6);
    b.wall_torch(Vec3::new(1.4, 1.6, HZ));
    furnish(b);
}

/// Adds the chimney, corner posts, a table and a bed to a cabin.
fn furnish(b: &mut Build) {
    b.cuboid(
        Vec3::new(HX - 0.02, 0.0, -0.5),
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
