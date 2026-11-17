//! The fishing camp: a thatched shelter, a drying rack with fish, a pulled-up boat and a fire.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::dress::{barrel, fire, on_ground};
use super::palette::{DARK_WOOD, PLANK, THATCH, WOOD};
use super::roofs::lean_height;
use super::tags::Tag;

/// Adds the lean-to shelter on four posts that reach up to its roof.
fn shelter(b: &mut Build) {
    let (lo, hi) = (Vec3::new(-2.6, 1.8, -2.7), Vec3::new(2.6, 1.8, 0.5));
    for (x, z) in [(-2.2, -2.4), (2.2, -2.4), (-2.2, 0.2), (2.2, 0.2)] {
        let (foot, top) = (Vec3::new(x, -0.3, z), lean_height(lo, hi, 1.0, z));
        let size = Vec2::new(0.09, top + 0.3);
        b.tagged(Tag::Post, |b| {
            b.pillar(foot, size, 4, Mat::solid(DARK_WOOD))
        });
    }
    b.lean_to(lo, hi, 1.0, Mat::deco(THATCH));
}

/// Adds the drying rack with two hanging fish.
fn rack(b: &mut Build) {
    for x in [-4.2, -2.6] {
        b.pillar(
            on_ground(b, x, 2.4) - Vec3::Y * 0.2,
            Vec2::new(0.07, 2.0),
            4,
            Mat::solid(DARK_WOOD),
        );
    }
    b.cuboid(
        Vec3::new(-4.3, 1.8, 2.34),
        Vec3::new(-2.5, 1.9, 2.46),
        Mat::deco(WOOD),
    );
    for x in [-3.8, -3.2] {
        let c = b.color([0.5, 0.65, 0.7], 1.0);
        let fish = [(0.0, 1.8), (0.12, 1.45), (0.0, 1.2), (-0.12, 1.45)];
        let p = fish.map(|(dx, y)| b.pt(Vec3::new(x + dx, y, 2.4)));
        let out = b.dir(Vec3::Z);
        b.sheet_quad(p, out, [c, c]);
    }
}

/// Adds a small rowing boat lying along local z with its bottom at height `y`.
fn boat(b: &mut Build, y: f32) {
    let (x0, x1, z0, z1) = (1.8, 3.2, 3.6, 7.0);
    b.cuboid(
        Vec3::new(x0, y, z0),
        Vec3::new(x1, y + 0.2, z1),
        Mat::solid(PLANK),
    );
    b.cuboid(
        Vec3::new(x0, y + 0.2, z0 - 0.02),
        Vec3::new(x0 + 0.14, y + 0.75, z1),
        Mat::deco(WOOD),
    );
    b.cuboid(
        Vec3::new(x1 - 0.14, y + 0.2, z0 - 0.02),
        Vec3::new(x1, y + 0.75, z1),
        Mat::deco(WOOD),
    );
    b.cuboid(
        Vec3::new(x0 + 0.14, y + 0.2, z0),
        Vec3::new(x1 - 0.14, y + 0.75, z0 + 0.14),
        Mat::deco(WOOD),
    );
}

/// Adds a shelter, a drying rack and a boat; local +z points to the water.
pub fn fishing(b: &mut Build) {
    b.settle(2.0, 0.0);
    shelter(b);
    rack(b);
    boat(b, on_ground(b, 2.5, 5.0).y);
    fire(b, on_ground(b, 0.0, 2.0), false);
    barrel(b, on_ground(b, 3.4, -0.8));
}
