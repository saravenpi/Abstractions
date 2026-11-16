//! Village houses, part one: the dispatcher and the small single-room kinds.

use bevy::prelude::*;

use super::detail::{chimney, furnish, windows};
use super::dressing::trim;
use super::lots::{House, Lot};
use super::shell::{Shell, entrance, plinth, walls};
use super::style::{Flavor, Style};
use super::{houses2, houses3};
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::dress::{barrel, crate_at, hay, on_ground};
use crate::world::structures::palette::{DARK_WOOD, WHITE};
use crate::world::structures::site::{Kind, Site};

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
        House::Cottage => cottage(b, &st, 2.8, 2.2),
        House::Long => cottage(b, &st, 4.6, 2.3),
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
    windows(b, s, st);
    trim(b, s, st);
}

/// Draws the style roof over a room with the given `rise`.
pub fn roof(b: &mut Build, st: &Style, s: &Shell, rise: f32) {
    let lo = Vec3::new(-s.hx - 0.5, s.h, -s.hz - 0.5);
    let hi = Vec3::new(s.hx + 0.5, s.h, s.hz + 0.5);
    let pick = b.roll(5);
    b.cap_roof(st, (lo, hi), rise, pick);
}

/// Draws a one-room house of half sizes `hx` by `hz`: the cottage and the long house.
fn cottage(b: &mut Build, st: &Style, hx: f32, hz: f32) {
    let s = Shell::new(hx, hz);
    room(b, st, &s);
    roof(b, st, &s, 1.6);
    if matches!(st.flavor, Flavor::Timber | Flavor::Taiga) && b.roll(2) < 0.7 {
        chimney(b, &s, s.h + 2.1);
    }
    furnish(b, &s, st);
    let base = on_ground(b, -hx + 0.8, hz + 1.4);
    barrel(b, base);
}

/// Draws a house with a covered porch on posts in front of the door.
fn porch(b: &mut Build, st: &Style) {
    let s = Shell {
        front: 1.6,
        ..Shell::new(3.2, 2.4)
    };
    room(b, st, &s);
    roof(b, st, &s, 1.7);
    let z = s.hz + s.front;
    for x in [-s.hx + 0.3, s.hx - 0.3] {
        b.pillar(
            Vec3::new(x, 0.0, z - 0.1),
            Vec2::new(0.12, 2.6),
            4,
            Mat::solid(DARK_WOOD),
        );
    }
    let (lo, hi) = (Vec3::new(-s.hx, 2.6, s.hz), Vec3::new(s.hx, 2.6, z + 0.35));
    b.lean_to(lo, hi, 0.55, Mat::deco(st.roof[0]));
    b.cuboid(
        Vec3::new(-s.hx + 0.5, 0.0, z - 0.3),
        Vec3::new(-s.hx + 1.8, 0.5, z + 0.1),
        Mat::deco(st.trim),
    );
    furnish(b, &s, st);
}

/// Draws a small storage shed with a lean-to roof and crates outside.
fn shed(b: &mut Build, st: &Style) {
    let s = Shell {
        h: 2.5,
        gap: 1.2,
        door_h: 2.2,
        ..Shell::new(1.9, 1.6)
    };
    room(b, st, &s);
    let (lo, hi) = (
        Vec3::new(-s.hx - 0.3, s.h, -s.hz - 0.3),
        Vec3::new(s.hx + 0.3, s.h, s.hz + 0.4),
    );
    b.lean_to(lo, hi, 0.7, Mat::deco(st.roof[0]));
    let base = on_ground(b, s.hx + 1.0, 0.4);
    crate_at(b, base, 0.8);
    let base = on_ground(b, s.hx + 1.0, -0.6);
    crate_at(b, base, 0.6);
    let base = on_ground(b, -s.hx - 1.0, 0.0);
    hay(b, base);
}

/// Draws the posts, back wall and counter of the market stall.
fn stall_frame(b: &mut Build, s: &Shell, st: &Style) {
    let corners = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
    for (sx, sz) in corners {
        let at = Vec3::new(sx * (s.hx - 0.2), 0.0, sz * (s.hz - 0.2));
        b.pillar(at, Vec2::new(0.08, 2.5), 4, Mat::deco(DARK_WOOD));
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
    let s = Shell {
        door: 1.5,
        gap: 1.2,
        ..Shell::new(2.4, 1.7)
    };
    plinth(b, &s, st);
    entrance(b, &s, st);
    stall_frame(b, &s, st);
    for i in 0..4 {
        let x = -s.hx - 0.2 + i as f32 * 1.25;
        let rgb = if i % 2 == 0 { st.cloth[0] } else { WHITE };
        let (lo, hi) = (
            Vec3::new(x, 2.4, -s.hz - 0.2),
            Vec3::new(x + 1.25, 2.4, s.hz + 0.5),
        );
        b.lean_to(lo, hi, 0.5, Mat::deco(rgb));
    }
    crate_at(b, Vec3::new(-1.6, 0.9, s.hz - 0.7), 0.4);
    crate_at(b, Vec3::new(-0.6, 0.9, s.hz - 0.7), 0.35);
    barrel(b, on_ground(b, s.hx + 0.9, 0.0));
}
