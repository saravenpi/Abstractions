//! The plaza center pieces: well, fountain, big tree with benches, and the market cross.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::layout::Centre;
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::on_ground;
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, STONE, STONE_DARK, STONE_LIGHT, WATER, WOOD};

/// Leaf green of the plaza tree.
const LEAF: [f32; 3] = [0.3, 0.52, 0.22];

/// Draws the center piece at the origin of the current frame.
pub fn draw(b: &mut Build, centre: Centre, st: &Style) {
    let site = b.site;
    match centre {
        Centre::Well => well(b, st),
        Centre::Fountain => fountain(b),
        Centre::TreeRing => tree_ring(b),
        Centre::Cross => cross(b),
    }
    if centre != Centre::Well {
        b.frame(Vec2::from_angle(0.4) * 5.4, 0.0);
        b.settle(1.6, 0.0);
        well(b, st);
    }
    b.begin(site);
    b.frame(Vec2::from_angle(0.4 + PI) * 5.4, 0.0);
    b.settle(1.0, 0.0);
    brazier(b);
    b.begin(site);
}

/// Draws a stone brazier on a stand with a lit fire.
fn brazier(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.1,
        Vec2::new(0.3, 0.9),
        6,
        Mat::solid(STONE_DARK),
    );
    b.pillar(Vec3::Y * 0.8, Vec2::new(0.55, 0.25), 8, Mat::deco(STONE));
    b.glow(LightKind::Brazier, Vec3::Y * 1.3);
}

/// Draws the crossbar, rope and bucket of the well.
fn bucket(b: &mut Build) {
    let beam = (Vec3::new(-1.0, 2.2, -0.04), Vec3::new(1.0, 2.3, 0.04));
    b.cuboid(beam.0, beam.1, Mat::deco(DARK_WOOD));
    b.cuboid(
        Vec3::new(-0.02, 1.7, -0.02),
        Vec3::new(0.02, 2.2, 0.02),
        Mat::deco(WOOD),
    );
    b.cuboid(
        Vec3::new(-0.2, 1.4, -0.2),
        Vec3::new(0.2, 1.7, 0.2),
        Mat::deco(WOOD),
    );
}

/// Draws a stone well: ring of stones, roof on two posts, a crossbar with rope and bucket, two torches.
fn well(b: &mut Build, st: &Style) {
    b.round_foundation(1.3, 8, 0.7, Mat::solid(STONE));
    b.spire(Vec3::Y * 0.72, Vec2::new(0.95, 0.001), 6, WATER);
    for x in [-1.0, 1.0] {
        b.pillar(
            Vec3::new(x, 0.7, 0.0),
            Vec2::new(0.1, 1.9),
            4,
            Mat::solid(DARK_WOOD),
        );
    }
    let (lo, hi) = (Vec3::new(-1.4, 2.6, -1.0), Vec3::new(1.4, 2.6, 1.0));
    b.gable(lo, hi, 0.9, Mat::deco(st.roof[0]));
    bucket(b);
    for x in [-1.9, 1.9] {
        let base = on_ground(b, x, 0.0);
        b.torch_post(base);
    }
}

/// Draws a round fountain with a central column and a jet.
fn fountain(b: &mut Build) {
    b.round_foundation(2.5, 10, 0.55, Mat::solid(STONE));
    b.spire(Vec3::Y * 0.57, Vec2::new(2.1, 0.001), 10, WATER);
    b.pillar(
        Vec3::Y * 0.55,
        Vec2::new(0.35, 1.7),
        6,
        Mat::solid(STONE_LIGHT),
    );
    b.pillar(Vec3::Y * 1.9, Vec2::new(0.9, 0.25), 8, Mat::deco(STONE));
    b.spire(Vec3::Y * 2.15, Vec2::new(0.5, 0.7), 6, WATER);
}

/// Draws one big tree with four benches and a ring of stones.
fn tree_ring(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.2,
        Vec2::new(0.55, 4.2),
        7,
        Mat::solid(DARK_WOOD),
    );
    let crown = b.color(LEAF, 1.0);
    let low = b.pt(Vec3::new(0.0, 5.2, 0.0));
    crate::world::solids::icosa(&mut b.mesh, low, Vec3::new(3.6, 2.4, 3.6), (0.2, 5), crown);
    let top = b.pt(Vec3::new(0.4, 6.8, 0.3));
    crate::world::solids::icosa(&mut b.mesh, top, Vec3::new(2.2, 1.6, 2.2), (0.2, 9), crown);
    for i in 0..4 {
        let a = i as f32 / 4.0 * TAU + 0.4;
        b.frame(Vec2::new(a.sin(), a.cos()) * 3.0, a);
        b.cuboid(
            Vec3::new(-0.9, -0.1, -0.22),
            Vec3::new(0.9, 0.42, 0.22),
            Mat::solid(WOOD),
        );
    }
    b.begin(b.site);
    for i in 0..10 {
        let a = i as f32 / 10.0 * TAU;
        b.pillar(
            Vec3::new(a.sin() * 4.4, -0.1, a.cos() * 4.4),
            Vec2::new(0.25, 0.35),
            5,
            Mat::deco(STONE_DARK),
        );
    }
}

/// Draws a three-tier round base with a tall cross-shaped post.
fn cross(b: &mut Build) {
    b.round_foundation(2.6, 8, 0.3, Mat::solid(STONE));
    b.pillar(
        Vec3::Y * 0.3,
        Vec2::new(1.8, 0.3),
        8,
        Mat::solid(STONE_LIGHT),
    );
    b.pillar(Vec3::Y * 0.6, Vec2::new(1.1, 0.3), 8, Mat::solid(STONE));
    b.cuboid(
        Vec3::new(-0.2, 0.9, -0.2),
        Vec3::new(0.2, 4.2, 0.2),
        Mat::solid(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(-0.8, 3.3, -0.12),
        Vec3::new(0.8, 3.65, 0.12),
        Mat::deco(DARK_WOOD),
    );
}
