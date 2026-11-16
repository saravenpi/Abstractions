//! Village houses, part three: the tower house and the village hall with its bell turret.

use bevy::prelude::*;

use super::detail::window;
use super::houses::room;
use super::shell::{Shell, T};
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::seat;
use crate::world::structures::palette::PLANK;

/// Draws a narrow tower house: a ground room under a tall windowed block and a pointed roof.
pub fn tower(b: &mut Build, st: &Style) {
    let s = Shell {
        h: 2.6,
        ..Shell::new(2.3, 2.3)
    };
    room(b, st, &s);
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
    b.cap_roof(
        st,
        (Vec3::new(-2.3, 7.4, -2.3), Vec3::new(2.3, 7.4, 2.3)),
        2.6,
        0.9,
    );
}

/// Draws the bell turret above the hall door.
fn turret(b: &mut Build, s: &Shell, st: &Style) {
    let z = s.hz - 0.2;
    b.cuboid(
        Vec3::new(-0.9, s.h, z - 1.6),
        Vec3::new(0.9, 7.0, z + 0.2),
        Mat::deco(st.wall[1]),
    );
    b.spire(
        Vec3::new(0.0, 7.0, z - 0.7),
        Vec2::new(1.3, 1.9),
        4,
        st.roof[0],
    );
    let corner = |x: f32, y: f32| Vec3::new(x, y, z + 0.22);
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
    let s = Shell {
        h: 3.4,
        gap: 2.0,
        door_h: 2.8,
        ..Shell::new(5.0, 3.6)
    };
    room(b, st, &s);
    let (lo, hi) = (
        Vec3::new(-s.hx - 0.5, s.h, -s.hz - 0.5),
        Vec3::new(s.hx + 0.5, s.h, s.hz + 0.5),
    );
    b.cap_roof(st, (lo, hi), 2.6, 0.0);
    turret(b, &s, st);
    seat(b, Vec3::new(-2.0, 0.0, 0.5), 2.4);
    seat(b, Vec3::new(2.0, 0.0, 0.5), 2.4);
    b.cuboid(
        Vec3::new(-1.0, 0.0, -s.hz + T),
        Vec3::new(1.0, 0.8, -s.hz + T + 0.8),
        Mat::solid(PLANK),
    );
}
