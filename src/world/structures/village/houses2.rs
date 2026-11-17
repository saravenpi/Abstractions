//! Village houses, part two: the two-storey house, the smithy and the barn.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::detail::{furnish, window};
use super::houses::{roof, room};
use super::houses3::ladder;
use super::interior::{light, table};
use super::lots::House;
use super::shell::{Shell, T};
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, hay, on_ground};
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, PLANK, STONE, STONE_DARK, THATCH, WATER};
use crate::world::structures::tags::Tag;

/// Floor height of the upper storey of the two-storey house.
const UPPER: f32 = 2.7;

/// Range along z of the upper doorway in the right wall.
const DOOR_Z: (f32, f32) = (-2.05, -0.85);

/// How far the smithy chimney stands out of the back wall inside the room.
const BREAST: f32 = 0.04;

/// Draws the upper walls, with a doorway in the right wall, and the upper floor planks.
fn upper_walls(b: &mut Build, s: &Shell, st: &Style) {
    let (hx, hz, top) = (s.hx, s.hz, UPPER + 2.5);
    let (d0, d1) = DOOR_Z;
    let list = [
        ([-hx, UPPER, -hz], [hx, top, -hz + T]),
        ([-hx, UPPER, -hz + T], [-hx + T, top, hz]),
        ([-hx + T, UPPER, hz - T], [hx - T, top, hz]),
        ([hx - T, UPPER, -hz + T], [hx, top, d0]),
        ([hx - T, UPPER, d1], [hx, top, hz - T]),
        ([hx - T, UPPER + 2.2, d0], [hx, top, d1]),
    ];
    b.tagged(Tag::Wall, |b| b.boxes(&list, Mat::solid(st.wall[0])));
    b.floor_pane(
        Vec2::new(-hx + T, -hz + T),
        Vec2::new(hx - T, hz - T),
        UPPER + 0.02,
        PLANK,
    );
    b.base_y += UPPER;
    window(b, Vec3::new(-hx, 0.0, 0.0), Vec3::NEG_X, st, true);
    window(b, Vec3::new(0.0, 0.0, hz), Vec3::Z, st, true);
    b.base_y -= UPPER;
}

/// Draws the outside stair along the right wall and its landing at the upper doorway.
fn outside_stair(b: &mut Build, s: &Shell, st: &Style) {
    let d1 = DOOR_Z.1;
    let low = b.ground_at(s.hx + 0.65, 3.0) - b.base_y;
    let run = (Vec2::new(s.hx + 0.65, d1 + 0.05), Vec2::Y);
    b.stair(run, 1.3, (low.min(-0.1), UPPER), Mat::solid(st.base));
    let (lo, hi) = (
        Vec3::new(s.hx, UPPER - 0.4, -s.hz - 0.5),
        Vec3::new(s.hx + 1.3, UPPER, d1 + 0.05),
    );
    b.cuboid(lo, hi, Mat::solid(st.trim));
}

/// Draws a house with an upper floor reached by an outside stair along the right wall.
pub fn two_story(b: &mut Build, st: &Style) {
    let s = House::TwoStory.shell();
    room(b, st, &s);
    let top = UPPER + 2.5;
    b.cuboid(
        Vec3::new(-s.hx, 2.5, -s.hz),
        Vec3::new(s.hx, UPPER, s.hz),
        Mat::solid(st.trim),
    );
    upper_walls(b, &s, st);
    outside_stair(b, &s, st);
    let pick = b.roll(5);
    b.cap_roof(st, s.tops(top), 1.7, pick);
    furnish(b, &s, st);
}

/// Draws the forge with its fire, the tall chimney behind it and a quench barrel beside it.
/// One collider covers the forge and the barrel.
fn forge(b: &mut Build, s: &Shell) {
    let (hx, hz) = (s.hx, s.hz);
    let lo = Vec3::new(hx - T - 1.5, 0.0, -hz + T);
    let hi = Vec3::new(hx - T, 1.0, -hz + T + 1.3);
    b.cuboid(lo, hi, Mat::deco(STONE));
    b.collider(lo - Vec3::X * 0.8, hi);
    b.glow(LightKind::Hearth, (lo + hi) * 0.5 + Vec3::Y * 0.8);
    b.cuboid(
        Vec3::new(hx - 2.2, 0.0, -hz - 0.8),
        Vec3::new(hx - 0.8, s.h + 2.6, -hz + T + BREAST),
        Mat::solid(STONE),
    );
    let tub = Vec3::new(lo.x - 0.42, 0.0, lo.z + 0.5);
    b.pillar(tub, Vec2::new(0.36, 0.9), 6, Mat::deco(DARK_WOOD));
    let water = Vec2::new(0.3, 0.02);
    b.pillar(tub + Vec3::Y * 0.9, water, 6, Mat::deco(WATER));
}

/// Draws the anvil on its stump, a workbench against the left wall and a lantern above the bench.
fn workshop(b: &mut Build, s: &Shell) {
    let stump = Vec2::new(0.35, 0.6);
    b.pillar(Vec3::new(-1.6, 0.0, -0.8), stump, 6, Mat::solid(DARK_WOOD));
    let anvil = [
        ([-1.75, 0.6, -0.9], [-1.45, 0.74, -0.7]),
        ([-2.0, 0.74, -0.95], [-1.2, 0.9, -0.65]),
    ];
    b.boxes(&anvil, Mat::deco(STONE_DARK));
    let x = -s.inner().x;
    table(b, Vec2::new(x, 0.2), FRAC_PI_2, 1.6);
    b.collider(Vec3::new(x, 0.0, -0.6), Vec3::new(x + 0.7, 0.8, 1.0));
    light(b, s, Vec3::new(x + 0.35, s.h, 0.2), true);
}

/// Draws a workshop with an open front, an anvil, a workbench, a forge and a tall chimney.
pub fn smithy(b: &mut Build, st: &Style) {
    let s = House::Smithy.shell();
    room(b, st, &s);
    roof(b, st, &s, 1.5);
    forge(b, &s);
    workshop(b, &s);
    let base = on_ground(b, -s.hx - 1.0, 0.5);
    barrel(b, base);
    let base = on_ground(b, -s.hx - 1.1, -0.8);
    crate_at(b, base, 0.7);
}

/// Draws a big barn with a wide door, a hay pile, a loft ladder and a hanging lantern.
pub fn barn(b: &mut Build, st: &Style) {
    let s = House::Barn.shell();
    let mut red = *st;
    red.wall[0] = [0.52, 0.17, 0.13];
    room(b, &red, &s);
    roof(b, &red, &s, 2.2);
    let z = -s.inner().y;
    let pile = [
        ([-3.2, 0.0, z + 0.05], [-2.1, 0.7, z + 0.85]),
        ([-2.05, 0.0, z + 0.05], [-0.95, 0.7, z + 0.85]),
        ([-2.6, 0.7, z + 0.1], [-1.5, 1.4, z + 0.8]),
    ];
    b.boxes(&pile, Mat::deco(THATCH));
    b.collider(Vec3::new(-3.2, 0.0, z), Vec3::new(-0.95, 0.7, z + 0.85));
    hay(b, Vec3::new(2.4, 0.0, -1.6));
    ladder(b, Vec3::new(1.2, 0.0, z), s.h - 0.3);
    light(b, &s, Vec3::new(0.0, s.h, 0.0), true);
    let base = on_ground(b, s.hx + 1.0, 1.0);
    hay(b, base);
    let base = on_ground(b, -s.hx - 1.0, 0.0);
    barrel(b, base);
}
