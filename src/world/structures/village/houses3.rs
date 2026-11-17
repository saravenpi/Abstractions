//! Village houses, part three: the tower house and the village hall with its bell turret.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::detail::window;
use super::houses::room;
use super::interior::{cupboard, light, seats, table};
use super::lots::House;
use super::shell::Shell;
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::palette::DARK_WOOD;

/// Draws a ladder `h` tall standing flat against the back wall, its foot centered on `at`.
pub fn ladder(b: &mut Build, at: Vec3, h: f32) {
    let wood = Mat::deco(DARK_WOOD);
    for x in [-0.25, 0.22] {
        let (lo, hi) = (Vec3::new(x, 0.0, 0.02), Vec3::new(x + 0.06, h, 0.1));
        b.cuboid(at + lo, at + hi, wood);
    }
    for i in 1..4 {
        let y = h * i as f32 * 0.25;
        let (lo, hi) = (Vec3::new(-0.19, y, 0.04), Vec3::new(0.22, y + 0.05, 0.08));
        b.cuboid(at + lo, at + hi, wood);
    }
}

/// Furnishes the tower room: a bookcase and a desk with a candle on the left wall, a stool, a ladder and a rug.
fn study(b: &mut Build, s: &Shell, st: &Style) {
    let (x, z) = (-s.inner().x, s.inner().y);
    cupboard(b, st, Vec2::new(x, -z + 0.5), FRAC_PI_2);
    table(b, Vec2::new(x, -0.25), FRAC_PI_2, 1.2);
    b.collider(Vec3::new(x, 0.0, -z), Vec3::new(x + 0.7, 0.9, 0.35));
    seats(b, Vec2::new(x + 0.85, -0.25), FRAC_PI_2, 0.4);
    light(b, s, Vec3::new(x + 0.35, 0.8, -0.55), false);
    ladder(b, Vec3::new(1.0, 0.0, -z), s.h);
    b.floor_pane(
        Vec2::new(-0.6, -0.7),
        Vec2::new(1.2, 0.9),
        0.05,
        st.cloth[2],
    );
}

/// Furnishes the hall: a long table between two benches, a sideboard on the back wall, a runner and one light.
fn banquet(b: &mut Build, s: &Shell, st: &Style) {
    let z = s.inner().y;
    table(b, Vec2::new(0.0, -0.35), 0.0, 3.6);
    b.collider(Vec3::new(-1.8, 0.0, -0.35), Vec3::new(1.8, 0.8, 0.35));
    seats(b, Vec2::new(0.0, 0.55), 0.0, 3.2);
    seats(b, Vec2::new(0.0, -0.85), 0.0, 3.2);
    for x in [-0.5, 0.5] {
        cupboard(b, st, Vec2::new(x, -z), 0.0);
    }
    b.collider(Vec3::new(-1.0, 0.0, -z), Vec3::new(1.0, 0.9, -z + 0.55));
    b.floor_pane(
        Vec2::new(-0.7, 1.1),
        Vec2::new(0.7, z - 0.2),
        0.05,
        st.cloth[0],
    );
    let hang = b.roll(23) < 0.5;
    let at = if hang {
        Vec3::Y * s.h
    } else {
        Vec3::new(0.9, 0.8, 0.0)
    };
    light(b, s, at, hang);
}

/// Draws a narrow tower house: a furnished ground room under a tall windowed block and a pointed roof.
pub fn tower(b: &mut Build, st: &Style) {
    let s = House::Tower.shell();
    room(b, st, &s);
    study(b, &s, st);
    b.cuboid(
        Vec3::new(-s.hx, 2.6, -s.hz),
        Vec3::new(s.hx, 3.0, s.hz),
        Mat::solid(st.trim),
    );
    b.cuboid(
        Vec3::new(-1.9, 3.0, -1.9),
        Vec3::new(1.9, 7.4, 1.9),
        Mat::deco(st.wall[0]),
    );
    b.base_y += 3.4;
    for out in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
        window(b, out * 1.9, out, st, false);
    }
    b.base_y -= 3.4;
    let block = (Vec3::new(-1.9, 7.4, -1.9), Vec3::new(1.9, 7.4, 1.9));
    b.cap_roof(st, block, 2.6, 0.9);
}

/// Draws the bell turret above the hall door, on a gallery floor that reaches the front and both end walls.
/// The turret stands a little out of the front wall, so that its face is not in the plane of the wall.
fn turret(b: &mut Build, s: &Shell, st: &Style) {
    let z = s.hz - 0.2;
    let Vec2 { x: gx, y: gz } = s.inner();
    b.cuboid(
        Vec3::new(-gx, s.h - 0.2, z - 1.8),
        Vec3::new(gx, s.h, gz),
        Mat::deco(st.trim),
    );
    b.cuboid(
        Vec3::new(-0.9, s.h, z - 1.6),
        Vec3::new(0.9, 7.0, z + 0.25),
        Mat::deco(st.wall[1]),
    );
    b.spire(
        Vec3::new(0.0, 7.0, z - 0.7),
        Vec2::new(1.3, 1.9),
        4,
        st.roof[0],
    );
    let corner = |x: f32, y: f32| Vec3::new(x, y, z + 0.27);
    b.pane(
        [
            corner(-0.45, 5.2),
            corner(0.45, 5.2),
            corner(0.45, 6.4),
            corner(-0.45, 6.4),
        ],
        Vec3::Z,
        [0.1, 0.12, 0.15],
    );
}

/// Draws the village hall: a long roofed room with a bell turret above the door.
pub fn hall(b: &mut Build, st: &Style) {
    let s = House::Hall.shell();
    room(b, st, &s);
    b.cap_roof(st, s.tops(s.h), 2.6, 0.0);
    turret(b, &s, st);
    banquet(b, &s, st);
}
