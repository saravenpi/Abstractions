//! The Cherry-grove shrine: a small hall, two stone lanterns and a torii gate.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{PLANK, STONE, STONE_DARK, STONE_LIGHT, TORII};

/// Adds a stone lantern standing at local `(x, z)` on the platform.
fn lantern(b: &mut Build, x: f32, z: f32) {
    b.cuboid(
        Vec3::new(x - 0.25, 0.0, z - 0.25),
        Vec3::new(x + 0.25, 0.8, z + 0.25),
        Mat::solid(STONE),
    );
    b.cuboid(
        Vec3::new(x - 0.4, 0.8, z - 0.4),
        Vec3::new(x + 0.4, 1.3, z + 0.4),
        Mat::solid(STONE_LIGHT),
    );
    b.gable(
        Vec3::new(x - 0.5, 1.3, z - 0.5),
        Vec3::new(x + 0.5, 1.3, z + 0.5),
        0.4,
        Mat::deco(STONE_DARK),
    );
}

/// Draws the torii gate seven meters in front of the hall.
fn gate(b: &mut Build) {
    for x in [-2.1, 2.1] {
        let low = b.ground_at(x, 7.0) - b.base_y - 1.0;
        b.pillar(
            Vec3::new(x, low, 7.0),
            Vec2::new(0.3, 4.5 - low),
            6,
            Mat::solid(TORII),
        );
    }
    b.boxes(
        &[
            ([-3.4, 4.3, 6.7], [3.4, 4.7, 7.3]),
            ([-2.1, 3.4, 6.8], [2.1, 3.7, 7.2]),
        ],
        Mat::solid(TORII),
    );
}

/// Height of the walls of the hall and of the eaves of its roof, in meters: a player, 1.8 m tall, walks
/// under the eaves up to the walls.
const HALL: f32 = 2.2;

/// Draws the platform, hall, lanterns and the gate.
pub fn shrine(b: &mut Build) {
    b.foundation(Vec2::splat(3.0), Mat::solid(STONE));
    b.boxes(&[([-1.4, 0.0, -1.2], [1.4, HALL, 1.0])], Mat::solid(PLANK));
    b.gable(
        Vec3::new(-2.1, HALL, -1.8),
        Vec3::new(2.1, HALL, 1.6),
        1.2,
        Mat::deco(STONE_DARK),
    );
    lantern(b, -2.2, 2.0);
    lantern(b, 2.2, 2.0);
    b.ramp(Vec3::new(0.0, 0.0, 3.0), 2.2);
    for x in [-1.9, 1.9] {
        b.torch_post(Vec3::new(x, 0.0, 3.4));
    }
    gate(b);
}
