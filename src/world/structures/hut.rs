//! The thatched hut on stilts.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{DARK_WOOD, PLANK, THATCH, WOOD};

/// Wall boxes of the hut: back, left, right, two door sides and the lintel.
const WALLS: [([f32; 3], [f32; 3]); 6] = [
    ([-2.2, 0.0, -1.8], [2.2, 2.2, -1.65]),
    ([-2.2, 0.0, -1.65], [-2.05, 2.2, 1.8]),
    ([2.05, 0.0, -1.65], [2.2, 2.2, 1.8]),
    ([-2.05, 0.0, 1.65], [-0.7, 2.2, 1.8]),
    ([0.7, 0.0, 1.65], [2.05, 2.2, 1.8]),
    ([-0.7, 2.0, 1.65], [0.7, 2.2, 1.8]),
];

/// Draws a raised hut with bamboo walls, a doorway, a thatch cone roof and a long ramp.
pub fn stilt_hut(b: &mut Build) {
    b.cuboid(
        Vec3::new(-2.6, -0.3, -2.2),
        Vec3::new(2.6, 0.0, 2.2),
        Mat::solid(PLANK),
    );
    for (x, z) in [
        (-2.3, -1.9),
        (2.3, -1.9),
        (-2.3, 0.0),
        (2.3, 0.0),
        (-2.3, 1.9),
        (2.3, 1.9),
    ] {
        let low = b.ground_at(x, z) - b.base_y - 0.6;
        b.pillar(
            Vec3::new(x, low, z),
            Vec2::new(0.18, -0.3 - low),
            5,
            Mat::deco(DARK_WOOD),
        );
    }
    b.boxes(&WALLS, Mat::solid(WOOD));
    b.spire(Vec3::Y * 2.2, Vec2::new(3.6, 2.2), 6, THATCH);
    b.ramp(Vec3::new(0.0, 0.0, 2.2), 1.4);
}
