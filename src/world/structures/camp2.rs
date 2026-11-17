//! The fishing, hunter and caravan camp layouts and the cooking tripod.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::dress::{barrel, crate_at, fire, on_ground, seat};
use super::palette::{CANVAS, DARK_WOOD, STONE_DARK, WOOD};
use super::roofs::lean_height;
use super::tags::Tag;
use crate::world::solids::{frustum, icosa};

/// Rug and awning stripe colors of the caravan.
const STRIPES: [[f32; 3]; 4] = [
    [0.78, 0.2, 0.18],
    [0.92, 0.85, 0.7],
    [0.2, 0.5, 0.62],
    [0.92, 0.85, 0.7],
];

/// Adds a three-legged tripod with a hanging pot, standing at the local point `at`.
pub fn tripod(b: &mut Build, at: Vec2) {
    let base = on_ground(b, at.x, at.y);
    let top = base + Vec3::Y * 1.7;
    let color = b.color(DARK_WOOD, 1.0);
    for i in 0..3 {
        let a = i as f32 * 2.094;
        let foot = base + Vec3::new(a.sin() * 0.8, 0.0, a.cos() * 0.8);
        let (from, to) = (b.pt(foot), b.pt(top));
        frustum(&mut b.mesh, from, to, Vec2::new(0.05, 0.04), color);
    }
    let pot = b.color(STONE_DARK, 1.0);
    let at = b.pt(top - Vec3::Y * 0.5);
    icosa(&mut b.mesh, at, Vec3::new(0.3, 0.25, 0.3), (0.1, 3), pot);
}

/// Adds the stacked log pile.
fn log_pile(b: &mut Build) {
    let rows = [
        (0.0, 0.0),
        (0.0, 0.5),
        (0.0, 1.0),
        (0.4, 0.25),
        (0.4, 0.75),
        (0.8, 0.5),
    ];
    for (i, (y, z)) in rows.into_iter().enumerate() {
        let base = on_ground(b, -2.0, -4.0 + z);
        let len = 1.06 + i as f32 * 0.035;
        let (lo, hi) = (
            Vec3::new(-len, y - 0.1, -0.22),
            Vec3::new(len, y + 0.4, 0.22),
        );
        b.cuboid(base + lo, base + hi, Mat::solid(WOOD));
    }
}

/// Adds a stump with an axe stuck in it.
fn stump(b: &mut Build) {
    let at = on_ground(b, 3.0, 3.0);
    b.pillar(
        at - Vec3::Y * 0.1,
        Vec2::new(0.4, 0.6),
        6,
        Mat::solid(DARK_WOOD),
    );
    let handle = (Vec3::new(-0.04, 0.5, -0.5), Vec3::new(0.04, 0.6, 0.3));
    b.cuboid(at + handle.0, at + handle.1, Mat::deco(WOOD));
    let head = (Vec3::new(-0.05, 0.48, 0.1), Vec3::new(0.05, 0.78, 0.4));
    b.cuboid(at + head.0, at + head.1, Mat::deco(STONE_DARK));
}

/// Adds a hide tent, a log pile and a stump with an axe.
pub fn hunter(b: &mut Build) {
    b.settle(2.0, 0.0);
    b.spire(Vec3::Y * -0.3, Vec2::new(2.1, 3.0), 6, [0.45, 0.33, 0.22]);
    let door = [
        Vec3::new(1.5, -0.3, -0.5),
        Vec3::new(1.5, -0.3, 0.5),
        Vec3::new(1.2, 1.2, 0.0),
    ];
    let color = b.color(DARK_WOOD, 0.6);
    let out = b.dir(Vec3::X);
    b.sheet_tri(door.map(|p| b.pt(p)), out, [color, color]);
    b.collider(Vec3::new(-1.2, -0.3, -1.2), Vec3::new(1.2, 1.5, 1.2));
    log_pile(b);
    stump(b);
    fire(b, on_ground(b, 3.4, -0.4), false);
    let base = on_ground(b, 3.4, 1.5);
    seat(b, base, 1.4);
}

/// Adds the striped awning on four poles that reach up to it.
fn awning(b: &mut Build) {
    let (lo, hi) = (Vec3::new(-2.9, 2.3, -2.1), Vec3::new(2.9, 2.3, 2.1));
    for (x, z) in [(-2.8, -1.9), (2.8, -1.9), (-2.8, 1.9), (2.8, 1.9)] {
        let (foot, top) = (Vec3::new(x, -0.3, z), lean_height(lo, hi, 0.5, z));
        let size = Vec2::new(0.09, top + 0.3);
        b.tagged(Tag::Post, |b| {
            b.pillar(foot, size, 4, Mat::solid(DARK_WOOD))
        });
    }
    b.awning(lo, hi, 0.5, &STRIPES);
}

/// Adds an awning on poles with crates, a barrel and two rugs.
pub fn caravan(b: &mut Build) {
    b.settle(3.0, 0.1);
    awning(b);
    b.floor_pane(Vec2::new(-2.0, -1.2), Vec2::new(0.2, 1.2), 0.03, STRIPES[0]);
    b.floor_pane(Vec2::new(0.6, -1.0), Vec2::new(2.2, 1.0), 0.03, STRIPES[2]);
    for (x, z, s) in [(-2.2, -1.4, 0.8), (-1.2, -1.5, 0.6), (2.0, -1.2, 0.7)] {
        let base = on_ground(b, x, z);
        crate_at(b, base, s);
    }
    barrel(b, on_ground(b, 2.3, 1.3));
    fire(b, on_ground(b, 5.0, 0.0), false);
    b.cuboid(
        Vec3::new(-0.4, 0.0, 1.4),
        Vec3::new(0.4, 0.5, 1.8),
        Mat::solid(CANVAS),
    );
}
