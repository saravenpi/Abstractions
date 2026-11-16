//! The campsite: two tents around a fire ring.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::{CANVAS, DARK_WOOD, FIRE, GLASS, STONE_DARK, WOOD};

/// Draws one tent whose open end faces local +x.
fn tent(b: &mut Build, off: Vec2, turn: f32) {
    b.frame(off, turn);
    b.settle(1.2, 0.0);
    b.gable(
        Vec3::new(-1.2, -0.7, -1.0),
        Vec3::new(1.2, -0.7, 1.0),
        2.3,
        Mat::deco(CANVAS),
    );
    let door = [
        Vec3::new(1.21, 0.0, -0.45),
        Vec3::new(1.21, 0.0, 0.45),
        Vec3::new(1.21, 1.1, 0.0),
    ];
    let color = b.color(DARK_WOOD, 0.6);
    let out = b.dir(Vec3::X);
    b.tri_out(door.map(|p| b.pt(p)), out, color);
    b.collider(Vec3::new(-1.1, -0.7, -0.9), Vec3::new(1.1, 0.9, 0.9));
}

/// Draws a stone-ringed fire, two log seats and two tents.
pub fn camp(b: &mut Build) {
    for i in 0..6 {
        let a = i as f32 / 6.0 * TAU;
        let at = Vec3::new(a.sin() * 1.1, -0.1, a.cos() * 1.1);
        b.pillar(at, Vec2::new(0.28, 0.4), 5, Mat::deco(STONE_DARK));
    }
    b.spire(Vec3::new(0.0, -0.1, 0.0), Vec2::new(0.4, 0.9), 5, FIRE);
    b.spire(Vec3::new(0.0, -0.1, 0.0), Vec2::new(0.25, 1.2), 5, GLASS);
    b.cuboid(
        Vec3::new(-0.9, -0.1, 1.7),
        Vec3::new(0.9, 0.35, 2.1),
        Mat::solid(WOOD),
    );
    b.cuboid(
        Vec3::new(-0.9, -0.1, -2.1),
        Vec3::new(0.9, 0.35, -1.7),
        Mat::solid(WOOD),
    );
    tent(b, Vec2::new(-3.6, 0.4), 0.2);
    tent(b, Vec2::new(3.6, -0.4), PI + 0.2);
}
