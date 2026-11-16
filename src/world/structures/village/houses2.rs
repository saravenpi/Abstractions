//! Village houses, part two: the two-storey house, smithy, barn, tower house and the hall.

use bevy::prelude::*;

use super::detail::{furnish, window};
use super::houses::{roof, room};
use super::shell::{Shell, T};
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, hay, on_ground};
use crate::world::structures::palette::{DARK_WOOD, PLANK, STONE, STONE_DARK};

/// Floor height of the upper storey of the two-storey house.
const UPPER: f32 = 2.7;

/// Range along z of the upper doorway in the right wall.
const DOOR_Z: (f32, f32) = (-2.05, -0.85);

/// Draws the upper walls, with a doorway in the right wall, and the upper floor planks.
fn upper_walls(b: &mut Build, s: &Shell, st: &Style) {
    let (hx, hz, top) = (s.hx, s.hz, UPPER + 2.5);
    let (d0, d1) = DOOR_Z;
    b.boxes(
        &[
            ([-hx, UPPER, -hz], [hx, top, -hz + T]),
            ([-hx, UPPER, -hz + T], [-hx + T, top, hz]),
            ([-hx + T, UPPER, hz - T], [hx - T, top, hz]),
            ([hx - T, UPPER, -hz + T], [hx, top, d0]),
            ([hx - T, UPPER, d1], [hx, top, hz - T]),
            ([hx - T, UPPER + 2.2, d0], [hx, top, d1]),
        ],
        Mat::solid(st.wall[0]),
    );
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
    let s = Shell {
        h: 2.5,
        door_h: 2.2,
        ..Shell::new(3.0, 2.4)
    };
    room(b, st, &s);
    let top = UPPER + 2.5;
    b.cuboid(
        Vec3::new(-s.hx, 2.5, -s.hz),
        Vec3::new(s.hx, UPPER, s.hz),
        Mat::solid(st.trim),
    );
    upper_walls(b, &s, st);
    outside_stair(b, &s, st);
    let (lo, hi) = (
        Vec3::new(-s.hx - 0.5, top, -s.hz - 0.5),
        Vec3::new(s.hx + 0.5, top, s.hz + 0.5),
    );
    let pick = b.roll(5);
    b.cap_roof(st, (lo, hi), 1.7, pick);
    furnish(b, &s, st);
}

/// Draws the forge, the tall chimney behind it, an anvil on a stump and goods outside.
fn forge(b: &mut Build, s: &Shell) {
    let (hx, hz) = (s.hx, s.hz);
    b.cuboid(
        Vec3::new(hx - T - 1.5, 0.0, -hz + T),
        Vec3::new(hx - T, 1.0, -hz + T + 1.3),
        Mat::solid(STONE),
    );
    b.cuboid(
        Vec3::new(hx - 2.2, 0.0, -hz - 0.7),
        Vec3::new(hx - 0.8, s.h + 2.6, -hz + T),
        Mat::solid(STONE),
    );
    b.pillar(
        Vec3::new(-1.6, 0.0, -0.8),
        Vec2::new(0.35, 0.6),
        6,
        Mat::solid(DARK_WOOD),
    );
    b.cuboid(
        Vec3::new(-2.0, 0.6, -1.0),
        Vec3::new(-1.2, 0.9, -0.6),
        Mat::deco(STONE_DARK),
    );
    let base = on_ground(b, -hx - 1.0, 0.5);
    barrel(b, base);
    let base = on_ground(b, -hx - 1.1, -0.8);
    crate_at(b, base, 0.7);
}

/// Draws a workshop with an open front, an anvil, a forge and a tall chimney.
pub fn smithy(b: &mut Build, st: &Style) {
    let s = Shell {
        door: 0.4,
        gap: 2.6,
        door_h: 2.4,
        ..Shell::new(3.4, 2.6)
    };
    room(b, st, &s);
    roof(b, st, &s, 1.5);
    forge(b, &s);
}

/// Draws a big barn with a wide door and hay inside and out.
pub fn barn(b: &mut Build, st: &Style) {
    let s = Shell {
        h: 3.4,
        gap: 2.6,
        door_h: 3.0,
        ..Shell::new(4.2, 3.2)
    };
    let mut red = *st;
    red.wall[0] = [0.52, 0.17, 0.13];
    room(b, &red, &s);
    roof(b, st, &s, 2.2);
    for (x, z) in [(-2.6, -2.0), (-1.4, -2.0), (2.4, -1.6)] {
        hay(b, Vec3::new(x, 0.0, z));
    }
    let base = on_ground(b, s.hx + 1.0, 1.0);
    hay(b, base);
    let base = on_ground(b, -s.hx - 1.0, 0.0);
    barrel(b, base);
}
