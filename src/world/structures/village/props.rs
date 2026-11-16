//! Props that close a street: lamp post, bench, hand cart and wayside shrine.

use bevy::prelude::*;

use super::layout::{End, Prop};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, on_ground, seat};
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, GLASS, PLANK, STONE, STONE_DARK, WOOD};

/// Draws a lamp post with a glowing lantern.
fn lamp(b: &mut Build) {
    b.pillar(
        Vec3::Y * -0.1,
        Vec2::new(0.09, 3.3),
        5,
        Mat::deco(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(-0.2, 3.2, -0.2),
        Vec3::new(0.2, 3.65, 0.2),
        Mat::deco(GLASS),
    );
    b.spire(Vec3::Y * 3.65, Vec2::new(0.3, 0.3), 4, STONE_DARK);
    b.glow(LightKind::Lamp, Vec3::Y * 3.4);
}

/// Draws a street lamp standing on the ground at the world point `pos`.
pub fn lamp_at(b: &mut Build, pos: Vec2) {
    b.stand(pos, 0.0);
    lamp(b);
}

/// Draws a bench with a backrest.
fn bench(b: &mut Build) {
    let base = on_ground(b, 0.0, 0.0);
    seat(b, base, 1.8);
    b.cuboid(
        Vec3::new(-0.9, 0.4, -0.3),
        Vec3::new(0.9, 0.9, -0.2),
        Mat::deco(WOOD),
    );
}

/// Draws a hand cart with two wheels, a handle, a barrel and a crate.
pub fn cart(b: &mut Build) {
    b.cuboid(
        Vec3::new(-0.8, 0.35, -0.5),
        Vec3::new(0.8, 0.75, 0.5),
        Mat::solid(PLANK),
    );
    b.pillar(
        Vec3::new(-0.9, 0.0, -0.5),
        Vec2::new(0.35, 0.1),
        8,
        Mat::deco(DARK_WOOD),
    );
    b.pillar(
        Vec3::new(-0.9, 0.0, 0.5),
        Vec2::new(0.35, 0.1),
        8,
        Mat::deco(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(0.8, 0.45, -0.05),
        Vec3::new(1.9, 0.52, 0.05),
        Mat::deco(DARK_WOOD),
    );
    barrel(b, on_ground(b, 1.4, 1.2));
    crate_at(b, on_ground(b, -1.4, 1.2), 0.6);
}

/// Draws a small stone wayside shrine under a gable.
fn shrine(b: &mut Build) {
    b.cuboid(
        Vec3::new(-0.7, -0.1, -0.7),
        Vec3::new(0.7, 0.5, 0.7),
        Mat::solid(STONE),
    );
    b.cuboid(
        Vec3::new(-0.45, 0.5, -0.45),
        Vec3::new(0.45, 1.5, 0.45),
        Mat::solid(STONE),
    );
    b.gable(
        Vec3::new(-0.8, 1.5, -0.8),
        Vec3::new(0.8, 1.5, 0.8),
        0.6,
        Mat::deco(STONE_DARK),
    );
}

/// Draws the prop that closes a street.
pub fn end_prop(b: &mut Build, end: &End) {
    b.stand(end.pos, end.yaw);
    for x in [-1.5, 1.5] {
        let base = on_ground(b, x, 0.3);
        b.torch_post(base);
    }
    match end.prop {
        Prop::Lamp => lamp(b),
        Prop::Bench => bench(b),
        Prop::Cart => cart(b),
        Prop::Shrine => shrine(b),
    }
}
