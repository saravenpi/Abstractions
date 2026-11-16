//! Small house details: windows with shutters, flower boxes, chimneys and the living room layout.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::interior::{bed, chest, cupboard, hearth, light, seats, table};
use super::shell::{Shell, T};
use super::style::{Flavor, Style};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::palette::STONE;

/// Dark glass.
const PANE: [f32; 3] = [0.1, 0.12, 0.15];

/// Draws one window recess on a wall: `at` is the wall point, `out` the wall normal along x or z.
pub fn window(b: &mut Build, at: Vec3, out: Vec3, st: &Style, shutters: bool) {
    let side = Vec3::new(out.z.abs(), 0.0, out.x.abs());
    let corner = |k: f32, y: f32| at + side * k + Vec3::Y * y + out * 0.02;
    let quad = [
        corner(-0.4, 1.0),
        corner(0.4, 1.0),
        corner(0.4, 1.9),
        corner(-0.4, 1.9),
    ];
    b.pane(quad, out, PANE);
    if shutters {
        for k in [-0.62, 0.62] {
            let m = at + side * k + out * 0.05;
            let half = side * 0.13 + out.abs() * 0.05;
            b.cuboid(
                m - half + Vec3::Y,
                m + half + Vec3::Y * 1.9,
                Mat::deco(st.trim),
            );
        }
    }
}

/// Draws a flower box under a front window centered at `x`.
fn flower_box(b: &mut Build, s: &Shell, st: &Style, x: f32) {
    let rgb = st.cloth[(b.site.variant as usize + x.abs() as usize) % 3];
    b.cuboid(
        Vec3::new(x - 0.4, 0.75, s.hz),
        Vec3::new(x + 0.4, 0.95, s.hz + 0.25),
        Mat::deco(rgb),
    );
}

/// Draws the windows of the front wall on both sides of the door.
fn front_windows(b: &mut Build, s: &Shell, st: &Style, shutters: bool) {
    let boxes = matches!(st.flavor, Flavor::Timber | Flavor::Cherry);
    let first = s.door + s.gap * 0.5 + 1.1;
    let slots = ((s.hx - 0.7 - first) / 1.9).ceil().max(0.0) as usize;
    for i in 0..slots {
        let x = first + i as f32 * 1.9;
        for sx in [x, 2.0 * s.door - x] {
            window(b, Vec3::new(sx, 0.0, s.hz), Vec3::Z, st, shutters);
            if boxes {
                flower_box(b, s, st, sx);
            }
        }
    }
}

/// Draws windows on both side walls and beside the door, with flower boxes on timber houses.
pub fn windows(b: &mut Build, s: &Shell, st: &Style) {
    let shutters = b.roll(11) < 0.6 && st.flavor != Flavor::Adobe;
    let rows = if s.hz > 2.4 { 2 } else { 1 };
    for i in 0..rows {
        let z = if rows == 1 {
            0.0
        } else {
            -s.hz * 0.45 + i as f32 * s.hz * 0.9
        };
        window(b, Vec3::new(s.hx, 0.0, z), Vec3::X, st, shutters);
        window(b, Vec3::new(-s.hx, 0.0, z), Vec3::NEG_X, st, shutters);
    }
    front_windows(b, s, st, shutters);
}

/// Draws a stone chimney against the right end wall, rising `top` above the floor, and its fireplace inside.
/// One collider covers the stack, the wall behind it and the fireplace.
pub fn chimney(b: &mut Build, s: &Shell, top: f32) {
    let (lo, hi) = (
        Vec3::new(s.hx, 0.0, -0.45),
        Vec3::new(s.hx + 0.8, top, 0.45),
    );
    b.cuboid(lo, hi, Mat::deco(STONE));
    b.collider(lo - Vec3::X * (T + 0.55), hi);
    hearth(b, s);
}

/// Draws the bed with a chest at its foot in a back corner, and a second bed in the long house.
/// `m` is 1 for a bed on the left wall and -1 for the mirrored room.
fn beds(b: &mut Build, s: &Shell, st: &Style, m: f32) {
    let Vec2 { x, y: z } = s.inner();
    bed(b, st, Vec2::new(-m * x, -z + 0.5), m * FRAC_PI_2);
    chest(b, st, Vec2::new(-m * (x - 2.0), -z + 0.5), m * FRAC_PI_2);
    let mut solids = vec![(-x, 2.53 - x)];
    if s.hx > 4.0 {
        bed(b, st, Vec2::new(m * x, -z + 0.5), -m * FRAC_PI_2);
        solids.push((x - 2.0, x));
    }
    for (x0, x1) in solids {
        let p = Vec3::new(m * x0, 0.0, -z);
        let q = Vec3::new(m * x1, 0.5, -z + 1.0);
        b.collider(p.min(q), p.max(q));
    }
}

/// Furnishes a living room along the back wall, mirrored by a roll: bed and chest, table with a seat,
/// cupboard, rug and one light on or above the table. The long house gets a long table and two beds.
pub fn furnish(b: &mut Build, s: &Shell, st: &Style) {
    let Vec2 { x, y: z } = s.inner();
    let m = if b.roll(21) < 0.5 { 1.0 } else { -1.0 };
    let long = s.hx > 4.0;
    let (tx, len) = if long { (0.0, 2.2) } else { (x - 1.65, 1.2) };
    let end = tx + len * 0.5 + 1.05;
    table(b, Vec2::new(m * tx, -z), 0.0, len);
    cupboard(b, st, Vec2::new(m * (end - 0.5), -z), 0.0);
    let p = Vec3::new(m * (tx - len * 0.5), 0.0, -z);
    let q = Vec3::new(m * end, 0.9, -z + 0.7);
    b.collider(p.min(q), p.max(q));
    let bench = if long || b.roll(22) < 0.5 {
        len * 0.8
    } else {
        0.4
    };
    seats(b, Vec2::new(m * tx, -z + 1.0), 0.0, bench);
    let rug = st.cloth[(b.roll(26) * 3.0) as usize % 3];
    b.floor_pane(
        Vec2::new(-0.9, -z + 1.5),
        Vec2::new(0.9, -z + 2.7),
        0.05,
        rug,
    );
    let hang = b.roll(23) < 0.5;
    let y = if hang { s.h } else { 0.8 };
    light(b, Vec3::new(m * (tx + len * 0.25), y, -z + 0.35), hang);
    beds(b, s, st, m);
}
