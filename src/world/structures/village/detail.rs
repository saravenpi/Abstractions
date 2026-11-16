//! Small house details: windows with shutters, flower boxes, chimneys, awnings, palms, lanterns, furniture.

use bevy::prelude::*;

use super::shell::{Shell, T};
use super::style::{Flavor, Style};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::palette::{DARK_WOOD, PLANK, STONE};

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

/// Draws a stone chimney against the right end wall, rising `top` above the floor.
pub fn chimney(b: &mut Build, s: &Shell, top: f32) {
    b.cuboid(
        Vec3::new(s.hx, 0.0, -0.45),
        Vec3::new(s.hx + 0.8, top, 0.45),
        Mat::solid(STONE),
    );
}

/// Draws a table with a stool and a bed at the back of the room.
pub fn furnish(b: &mut Build, s: &Shell, st: &Style) {
    let (x0, z0) = (-s.hx + T + 0.3, -s.hz + T + 0.3);
    b.cuboid(
        Vec3::new(x0, 0.0, z0),
        Vec3::new(x0 + 1.1, 0.8, z0 + 0.8),
        Mat::solid(PLANK),
    );
    b.cuboid(
        Vec3::new(x0 + 1.5, 0.0, z0),
        Vec3::new(x0 + 1.9, 0.45, z0 + 0.4),
        Mat::deco(DARK_WOOD),
    );
    let bed = Vec3::new(s.hx - T - 1.1, 0.0, -s.hz + T);
    b.cuboid(bed, bed + Vec3::new(1.1, 0.5, 2.0), Mat::deco(st.cloth[0]));
}
