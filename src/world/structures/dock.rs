//! The wooden pier that reaches from a beach over the water.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{DARK_WOOD, PLANK};

/// Draws a plank walkway on posts with a wider head, running along local +z.
pub fn dock(b: &mut Build) {
    for i in 0..7 {
        let z = i as f32 * 2.0;
        b.cuboid(
            Vec3::new(-1.1, -0.25, z),
            Vec3::new(1.1, 0.0, z + 1.96),
            Mat::solid(PLANK),
        );
    }
    b.cuboid(
        Vec3::new(-2.4, -0.25, 14.0),
        Vec3::new(2.4, 0.0, 16.0),
        Mat::solid(PLANK),
    );
    for x in [-2.1, 2.1] {
        b.torch_post(Vec3::new(x, 0.0, 15.6));
    }
    for i in 0..6 {
        let z = 0.3 + i as f32 * 2.8;
        for x in [-1.0, 1.0] {
            let low = b.ground_at(x, z) - b.base_y - 0.6;
            b.pillar(
                Vec3::new(x, low, z),
                Vec2::new(0.16, 0.35 - low),
                5,
                Mat::solid(DARK_WOOD),
            );
        }
    }
}
