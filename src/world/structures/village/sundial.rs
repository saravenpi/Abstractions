//! The plaza sundial: a stepped stone base, a low pedestal with a dial, a slanted gnomon and four hour stones.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::on_ground;
use crate::world::structures::palette::{STONE, STONE_DARK, STONE_LIGHT};

/// Dark bronze of the gnomon.
const BRONZE: [f32; 3] = [0.4, 0.28, 0.14];

/// Draws the gnomon on the dial at height `y`: a thin wedge that rises toward local +z.
fn gnomon(b: &mut Build, y: f32) {
    let side = |x: f32| [(0.0, -0.6), (0.0, 0.5), (0.55, 0.5)].map(|(h, z)| Vec3::new(x, y + h, z));
    let (l, r) = (side(-0.04), side(0.04));
    for (tri, out) in [(l, Vec3::NEG_X), (r, Vec3::X)] {
        let color = b.color(BRONZE, 0.9);
        b.tri_out(tri.map(|c| b.pt(c)), b.dir(out), color);
    }
    b.pane([l[0], r[0], r[2], l[2]], Vec3::new(0.0, 1.1, -0.55), BRONZE);
    b.pane([l[1], r[1], r[2], l[2]], Vec3::Z, BRONZE);
}

/// Draws a three-tier round base with a pedestal, a dial disc, its gnomon and four hour stones on the ground.
pub fn sundial(b: &mut Build) {
    b.round_foundation(2.6, 8, 0.3, Mat::solid(STONE));
    b.pillar(
        Vec3::Y * 0.3,
        Vec2::new(1.8, 0.3),
        8,
        Mat::solid(STONE_LIGHT),
    );
    b.pillar(Vec3::Y * 0.6, Vec2::new(1.1, 0.3), 8, Mat::solid(STONE));
    b.pillar(
        Vec3::Y * 0.9,
        Vec2::new(0.45, 0.4),
        6,
        Mat::solid(STONE_DARK),
    );
    b.pillar(
        Vec3::Y * 1.3,
        Vec2::new(0.8, 0.08),
        12,
        Mat::deco(STONE_LIGHT),
    );
    gnomon(b, 1.38);
    for i in 0..4 {
        let a = i as f32 / 4.0 * TAU;
        let at = on_ground(b, a.sin() * 3.3, a.cos() * 3.3) - Vec3::Y * 0.1;
        b.pillar(at, Vec2::new(0.22, 0.5), 5, Mat::deco(STONE_DARK));
    }
}
