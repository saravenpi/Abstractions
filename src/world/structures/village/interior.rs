//! Low-poly furniture of village houses: table, seats, bed, cupboard, chest, hearth and the room light.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::shell::Shell;
use super::style::Style;
use crate::world::structures::brush::{Build, Mat};
use crate::world::structures::glow::LightKind;
use crate::world::structures::palette::{DARK_WOOD, GLASS, PLANK, STONE, STONE_LIGHT, WHITE, WOOD};

/// Pale mattress linen.
const LINEN: [f32; 3] = [0.87, 0.83, 0.74];

/// Fired clay of bowls and pots.
const CLAY: [f32; 3] = [0.72, 0.42, 0.28];

/// One box of a furniture piece in the piece frame: low corner, high corner and color.
type Slab = ([f32; 3], [f32; 3], [f32; 3]);

/// Draws `list` as walk-through boxes in a frame at `at` turned by `turn`, whose +z points into the
/// room, then returns to the house frame. The caller adds the collider of the large pieces.
fn put(b: &mut Build, at: Vec2, turn: f32, list: &[Slab]) {
    b.frame(at, turn);
    for (lo, hi, rgb) in list {
        b.cuboid(
            Vec3::from_array(*lo),
            Vec3::from_array(*hi),
            Mat::deco(*rgb),
        );
    }
    b.frame(Vec2::ZERO, 0.0);
}

/// Draws a table `len` long and 0.7 m deep with its back edge centered on `at`: a top on four legs and a bowl.
pub fn table(b: &mut Build, at: Vec2, turn: f32, len: f32) {
    let x = len * 0.5;
    let leg = |sx: f32, z: f32| {
        let c = sx * (x - 0.1);
        (
            [c - 0.04, 0.0, z - 0.04],
            [c + 0.04, 0.72, z + 0.04],
            DARK_WOOD,
        )
    };
    let bowl = (
        [-x * 0.5 - 0.1, 0.8, 0.25],
        [-x * 0.5 + 0.1, 0.88, 0.45],
        CLAY,
    );
    let top = ([-x, 0.72, 0.0], [x, 0.8, 0.7], PLANK);
    let legs = [leg(-1.0, 0.1), leg(1.0, 0.1), leg(-1.0, 0.6), leg(1.0, 0.6)];
    put(
        b,
        at,
        turn,
        &[top, legs[0], legs[1], legs[2], legs[3], bowl],
    );
}

/// Draws a bench `len` long, or a stool when short, with its back edge centered on `at`.
pub fn seats(b: &mut Build, at: Vec2, turn: f32, len: f32) {
    let x = len * 0.5;
    let leg = |sx: f32| {
        let c = sx * (x - 0.08);
        ([c - 0.03, 0.0, 0.03], [c + 0.03, 0.4, 0.27], DARK_WOOD)
    };
    let top = ([-x, 0.4, 0.0], [x, 0.46, 0.3], PLANK);
    put(b, at, turn, &[top, leg(-1.0), leg(1.0)]);
}

/// Draws a 1 m by 2 m bed with its headboard centered on `at`: frame, mattress, blanket and pillow.
pub fn bed(b: &mut Build, st: &Style, at: Vec2, turn: f32) {
    let cloth = st.cloth[(b.roll(24) * 3.0) as usize % 3];
    let list = [
        ([-0.5, 0.0, 0.0], [0.5, 0.95, 0.08], st.trim),
        ([-0.5, 0.0, 0.08], [0.5, 0.32, 2.0], WOOD),
        ([-0.45, 0.32, 0.1], [0.45, 0.46, 1.96], LINEN),
        ([-0.47, 0.32, 0.68], [0.47, 0.5, 1.98], cloth),
        ([-0.3, 0.46, 0.16], [0.3, 0.56, 0.5], WHITE),
    ];
    put(b, at, turn, &list);
}

/// Draws a 1 m wide cupboard with its back centered on `at`, with two or three pots on top.
pub fn cupboard(b: &mut Build, st: &Style, at: Vec2, turn: f32) {
    let pot =
        |x: f32, h: f32, rgb: [f32; 3]| ([x - 0.09, 0.95, 0.2], [x + 0.09, 0.95 + h, 0.38], rgb);
    let list = [
        ([-0.5, 0.0, 0.0], [0.5, 0.9, 0.55], PLANK),
        ([-0.5, 0.9, 0.0], [0.5, 0.95, 0.6], st.trim),
        pot(-0.3, 0.22, CLAY),
        pot(0.28, 0.14, st.cloth[1]),
        pot(0.0, 0.3, STONE_LIGHT),
    ];
    let n = if b.roll(25) < 0.5 { 4 } else { 5 };
    put(b, at, turn, &list[..n]);
}

/// Draws a chest with its back centered on `at`: a body under a wider lid.
pub fn chest(b: &mut Build, st: &Style, at: Vec2, turn: f32) {
    let body = ([-0.4, 0.0, 0.03], [0.4, 0.36, 0.5], WOOD);
    let lid = ([-0.43, 0.36, 0.0], [0.43, 0.5, 0.53], st.trim);
    put(b, at, turn, &[body, lid]);
}

/// Draws an open stone fireplace on the inside of the right end wall, with a fire burning in its mouth.
pub fn hearth(b: &mut Build, s: &Shell) {
    let x = s.inner().x;
    let list = [
        ([-0.6, 0.0, 0.0], [-0.4, 1.1, 0.55], STONE),
        ([0.4, 0.0, 0.0], [0.6, 1.1, 0.55], STONE),
        ([-0.7, 1.1, 0.0], [0.7, 1.3, 0.65], STONE_LIGHT),
        ([-0.25, 0.0, 0.2], [0.25, 0.12, 0.44], DARK_WOOD),
    ];
    put(b, Vec2::new(x, 0.0), -FRAC_PI_2, &list);
    b.glow(LightKind::Hearth, Vec3::new(x - 0.32, 0.45, 0.0));
}

/// Draws the room light: when `hang`, a lantern on a chain under a tie beam that spans the room of `s` from the
/// back wall to the front wall at the height of `at`, else a candle standing on `at`.
/// The lantern glass closes around its flame, so the light only reaches the room.
pub fn light(b: &mut Build, s: &Shell, at: Vec3, hang: bool) {
    let half = Vec3::new(0.1, 0.0, 0.1);
    if hang {
        let (beam, z) = (Vec3::new(0.07, 0.14, 0.0), Vec3::Z * s.inner().y);
        let top = Vec3::new(at.x, at.y - 0.005, 0.0);
        b.cuboid(
            top - beam - z,
            top + Vec3::X * beam.x + z,
            Mat::deco(DARK_WOOD),
        );
        let chain = Vec2::new(0.02, 0.4);
        b.pillar(at - Vec3::Y * 0.4, chain, 4, Mat::deco(DARK_WOOD));
        b.cuboid(
            at - half - Vec3::Y * 0.74,
            at + half - Vec3::Y * 0.4,
            Mat::deco(GLASS),
        );
        b.glow(LightKind::Candle, at - Vec3::Y * 0.57);
    } else {
        b.cuboid(
            at - half * 0.3,
            at + half * 0.3 + Vec3::Y * 0.18,
            Mat::deco(WHITE),
        );
        b.glow(LightKind::Candle, at + Vec3::Y * 0.33);
    }
}
