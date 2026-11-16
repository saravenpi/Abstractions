//! The stepped sandstone pyramid.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{SAND, SAND_DARK, SAND_LIGHT};

/// Height of every pyramid step, within what the player can climb.
const STEP: f32 = 0.28;

/// Draws twelve climbable tiers topped by a small pillared pavilion.
pub fn pyramid(b: &mut Build) {
    b.foundation(Vec2::splat(9.6), Mat::solid(SAND_DARK));
    for i in 0..12 {
        let half = 9.0 - 0.5 * i as f32;
        let y = STEP * i as f32;
        let rgb = if i % 2 == 0 { SAND } else { SAND_LIGHT };
        b.cuboid(
            Vec3::new(-half, y, -half),
            Vec3::new(half, y + STEP, half),
            Mat::solid(rgb),
        );
    }
    let top = STEP * 12.0;
    for (x, z) in [(-2.8, -2.8), (2.8, -2.8), (-2.8, 2.8), (2.8, 2.8)] {
        b.pillar(
            Vec3::new(x, top, z),
            Vec2::new(0.3, 2.8),
            4,
            Mat::solid(SAND_DARK),
        );
    }
    b.cuboid(
        Vec3::new(-3.4, top + 2.8, -3.4),
        Vec3::new(3.4, top + 3.2, 3.4),
        Mat::solid(SAND_DARK),
    );
    b.spire(Vec3::Y * (top + 3.2), Vec2::new(3.0, 1.8), 4, SAND);
    b.ramp(Vec3::new(0.0, 0.0, 9.7), 4.0);
}
