//! Village houses, part one: the dispatcher and the small single-room kinds.

use bevy::prelude::*;

use super::detail::{chimney, furnish, windows};
use super::dressing::trim;
use super::lots::{House, Lot};
use super::shell::{Shell, entrance, frame, plinth, walls};
use super::style::{Flavor, Style};
use super::{houses2, houses3};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, hay, on_ground};
use crate::world::structures::palette::{CANVAS, DARK_WOOD, WHITE};
use crate::world::structures::roofs::lean_height;
use crate::world::structures::site::{Kind, Site};
use crate::world::structures::tags::Tag;

/// How much higher the back edge of the stall awning is than its front edge.
const AWNING_RISE: f32 = 0.5;

/// Moves the builder to `lot` and draws its house, with the wall and roof tones picked from the lot.
pub fn draw(b: &mut Build, village: &Style, lot: &Lot) {
    b.begin(Site {
        kind: Kind::Village,
        origin: lot.pos,
        yaw: lot.yaw,
        floor: lot.floor,
        depth: lot.depth,
        variant: lot.variant,
    });
    let mut st = *village;
    if lot.variant & 1 != 0 {
        st.wall.swap(0, 1);
    }
    if lot.variant & 2 != 0 {
        st.roof.swap(0, 1);
    }
    match lot.house {
        House::Cottage | House::Long => cottage(b, &st, &lot.house.shell()),
        House::Porch => porch(b, &st),
        House::Shed => shed(b, &st),
        House::Stall => stall(b, &st),
        House::TwoStory => houses2::two_story(b, &st),
        House::Smithy => houses2::smithy(b, &st),
        House::Barn => houses2::barn(b, &st),
        House::Tower => houses3::tower(b, &st),
        House::Hall => houses3::hall(b, &st),
    }
}

/// Draws plinth, walls, stairs, windows and door trim of a room.
pub fn room(b: &mut Build, st: &Style, s: &Shell) {
    plinth(b, s, st);
    walls(b, s, Mat::solid(st.wall[0]));
    entrance(b, s, st);
    frame(b, s, st);
    windows(b, s, st);
    trim(b, s, st);
}

/// Draws the style roof on the walls of a room with the given `rise`.
pub fn roof(b: &mut Build, st: &Style, s: &Shell, rise: f32) {
    let pick = b.roll(5);
    b.cap_roof(st, s.tops(s.h), rise, pick);
}

/// Draws the one-room house of shell `s`, the cottage or the long house, with a hearth under its chimney.
fn cottage(b: &mut Build, st: &Style, s: &Shell) {
    room(b, st, s);
    roof(b, st, s, 1.6);
    if matches!(st.flavor, Flavor::Timber | Flavor::Taiga) && b.roll(2) < 0.7 {
        chimney(b, s, s.h + 2.1);
    }
    furnish(b, s, st);
    let base = on_ground(b, -s.hx + 0.8, s.hz + 1.4);
    barrel(b, base);
}

/// Draws a house with a covered porch in front of the door. The porch roof passes under the eaves of the
/// house roof, through their edge, and its posts reach up to it.
fn porch(b: &mut Build, st: &Style) {
    let s = House::Porch.shell();
    room(b, st, &s);
    roof(b, st, &s, 1.7);
    let z = s.hz + s.front;
    let (rise, run) = (0.4, s.front + 0.35);
    let low = s.h + 0.5 * rise / run - rise;
    let (lo, hi) = (Vec3::new(-s.hx, low, s.hz), Vec3::new(s.hx, low, z + 0.35));
    let top = lean_height(lo, hi, rise, z - 0.1);
    for x in [-s.hx + 0.3, s.hx - 0.3] {
        let (foot, size) = (Vec3::new(x, 0.0, z - 0.1), Vec2::new(0.12, top));
        b.tagged(Tag::Post, |b| {
            b.pillar(foot, size, 4, Mat::solid(DARK_WOOD))
        });
    }
    b.lean_to(lo, hi, rise, Mat::deco(st.roof[0]));
    b.cuboid(
        Vec3::new(-s.hx + 0.5, 0.0, z - 0.3),
        Vec3::new(-s.hx + 1.8, 0.5, z + 0.1),
        Mat::solid(st.trim),
    );
    furnish(b, &s, st);
}

/// Draws a small storage shed with a lean-to roof, two sacks inside and crates outside.
fn shed(b: &mut Build, st: &Style) {
    let s = House::Shed.shell();
    room(b, st, &s);
    let (lo, hi) = (
        Vec3::new(-s.hx - 0.3, s.h, -s.hz - 0.3),
        Vec3::new(s.hx + 0.3, s.h, s.hz + 0.4),
    );
    b.lean_over(
        s.tops(s.h),
        (lo, hi),
        0.7,
        (Mat::deco(st.roof[0]), st.wall[0]),
    );
    for (x, z) in [(-1.2, -0.9), (-0.6, -0.95)] {
        let sack = Vec2::new(0.26, 0.5);
        b.pillar(Vec3::new(x, 0.0, z), sack, 6, Mat::solid(CANVAS));
    }
    let base = on_ground(b, s.hx + 1.0, 0.4);
    crate_at(b, base, 0.8);
    let base = on_ground(b, s.hx + 1.0, -0.6);
    crate_at(b, base, 0.6);
    let base = on_ground(b, -s.hx - 1.0, 0.0);
    hay(b, base);
}

/// Draws the back wall and counter of the market stall, and its four posts up to the awning over `roof`.
fn stall_frame(b: &mut Build, s: &Shell, st: &Style, roof: (Vec3, Vec3)) {
    let corners = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
    for (sx, sz) in corners {
        let at = Vec3::new(sx * (s.hx - 0.2), 0.0, sz * (s.hz - 0.2));
        let size = Vec2::new(0.08, lean_height(roof.0, roof.1, AWNING_RISE, at.z));
        b.tagged(Tag::Post, |b| b.pillar(at, size, 4, Mat::solid(DARK_WOOD)));
    }
    let back = (
        Vec3::new(-s.hx, 0.0, -s.hz),
        Vec3::new(s.hx, 2.2, -s.hz + 0.15),
    );
    b.cuboid(back.0, back.1, Mat::solid(st.wall[1]));
    let counter = (
        Vec3::new(-s.hx + 0.3, 0.0, s.hz - 1.0),
        Vec3::new(0.7, 0.9, s.hz - 0.4),
    );
    b.cuboid(counter.0, counter.1, Mat::solid(st.wall[0]));
}

/// Draws an open market stall: platform, counter, striped awning and goods.
fn stall(b: &mut Build, st: &Style) {
    let s = House::Stall.shell();
    plinth(b, &s, st);
    entrance(b, &s, st);
    let x = -s.hx - 0.2;
    let roof = (
        Vec3::new(x, 2.4, -s.hz - 0.2),
        Vec3::new(x + 5.0, 2.4, s.hz + 0.5),
    );
    stall_frame(b, &s, st, roof);
    let stripes = [st.cloth[0], WHITE, st.cloth[0], WHITE];
    b.awning(roof.0, roof.1, AWNING_RISE, &stripes);
    crate_at(b, Vec3::new(-1.6, 0.9, s.hz - 0.7), 0.4);
    crate_at(b, Vec3::new(-0.6, 0.9, s.hz - 0.7), 0.35);
    barrel(b, on_ground(b, s.hx + 0.9, 0.0));
}
