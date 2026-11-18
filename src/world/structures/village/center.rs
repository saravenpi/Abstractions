//! The plaza center pieces: well, fountain and big tree with benches, each settled on its own ground.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::layout::Centre;
use super::style::Style;
use super::sundial::sundial;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::on_ground;
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, STONE, STONE_DARK, STONE_LIGHT, WATER, WOOD};
use crate::world::structures::tags::Tag;

/// Leaf green of the plaza tree.
const LEAF: [f32; 3] = [0.3, 0.52, 0.22];

/// Draws the center piece at the origin of the current frame, raised to the highest ground under it.
pub fn draw(b: &mut Build, centre: Centre, st: &Style) {
    let site = b.site;
    let reach = match centre {
        Centre::TreeRing => 0.0,
        Centre::Well => 1.6,
        Centre::Fountain | Centre::Sundial => 2.7,
    };
    b.settle(reach, 0.0);
    match centre {
        Centre::Well => well(b, st),
        Centre::Fountain => fountain(b),
        Centre::TreeRing => tree_ring(b),
        Centre::Sundial => sundial(b),
    }
    b.begin(site);
    if centre != Centre::Well {
        b.frame(Vec2::from_angle(0.4) * 5.4, 0.0);
        b.settle(1.6, 0.0);
        well(b, st);
    }
    b.begin(site);
    b.frame(Vec2::from_angle(0.4 + PI) * 5.4, 0.0);
    b.settle(0.3, 0.0);
    brazier(b);
    b.begin(site);
}

/// Draws a stone brazier on a stand with a lit fire.
fn brazier(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.3,
        Vec2::new(0.3, 1.1),
        6,
        Mat::solid(STONE_DARK),
    );
    b.pillar(Vec3::Y * 0.8, Vec2::new(0.55, 0.25), 8, Mat::solid(STONE));
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
        Mat::solid(WOOD),
    );
}

/// Draws a stone well: ring of stones, a roof whose ridge rests on two posts, a crossbar with rope and bucket,
/// two torches.
fn well(b: &mut Build, st: &Style) {
    b.round_foundation(1.3, 8, 0.7, Mat::solid(STONE));
    b.spire(Vec3::Y * 0.72, Vec2::new(0.95, 0.001), 6, WATER);
    for x in [-1.0, 1.0] {
        let (foot, size) = (Vec3::new(x, 0.7, 0.0), Vec2::new(0.1, 2.8));
        b.tagged(Tag::Post, |b| {
            b.pillar(foot, size, 4, Mat::solid(DARK_WOOD))
        });
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
    b.pillar(Vec3::Y * 1.9, Vec2::new(0.9, 0.25), 8, Mat::solid(STONE));
    b.spire(Vec3::Y * 2.15, Vec2::new(0.5, 0.7), 6, WATER);
}

/// Draws bench `i` of the tree ring on its own ground, and returns the feet of its two legs.
fn bench(b: &mut Build, i: u32) -> [Vec3; 2] {
    let a = i as f32 / 4.0 * TAU + 0.4;
    b.frame(Vec2::new(a.sin(), a.cos()) * 3.0, a);
    b.settle(0.7, 0.0);
    let feet = [-0.7, 0.7].map(|x| on_ground(b, x, 0.0) - Vec3::Y * 0.15);
    for foot in feet {
        let (lo, hi) = (Vec3::new(-0.08, 0.0, -0.2), Vec3::new(0.08, 0.0, 0.2));
        b.cuboid(
            foot + lo,
            Vec3::new(foot.x, 0.3, 0.0) + hi,
            Mat::deco(DARK_WOOD),
        );
    }
    let (lo, hi) = (Vec3::new(-0.9, 0.3, -0.22), Vec3::new(0.9, 0.42, 0.22));
    b.cuboid(lo, hi, Mat::deco(WOOD));
    b.collider(Vec3::new(lo.x, -0.1, lo.z), hi);
    feet
}

/// Returns the world position of every leg foot of the four tree-ring benches.
#[cfg(test)]
pub fn bench_feet(b: &mut Build) -> Vec<Vec3> {
    let world = Vec3::new(b.center.x, 0.0, b.center.y);
    let feet = |i| bench(b, i).map(|f| b.pt(f) + world);
    (0..4).flat_map(feet).collect()
}

/// Draws one big tree with four benches and a ring of stones, the benches and stones each on their own ground.
fn tree_ring(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.4,
        Vec2::new(0.55, 4.4),
        7,
        Mat::solid(DARK_WOOD),
    );
    let crown = b.color(LEAF, 1.0);
    let low = b.pt(Vec3::new(0.0, 5.2, 0.0));
    let top = b.pt(Vec3::new(0.4, 6.8, 0.3));
    b.tagged(Tag::Soft, |b| {
        crate::world::solids::icosa(&mut b.mesh, low, Vec3::new(3.6, 2.4, 3.6), (0.2, 5), crown);
        crate::world::solids::icosa(&mut b.mesh, top, Vec3::new(2.2, 1.6, 2.2), (0.2, 9), crown);
    });
    for i in 0..4 {
        bench(b, i);
    }
    b.begin(b.site);
    for i in 0..10 {
        let a = i as f32 / 10.0 * TAU;
        b.pillar(
            on_ground(b, a.sin() * 4.4, a.cos() * 4.4) - Vec3::Y * 0.1,
            Vec2::new(0.25, 0.35),
            5,
            Mat::solid(STONE_DARK),
        );
    }
}
