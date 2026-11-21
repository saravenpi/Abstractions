//! The stones stood on rings: basalt, a few blocky columns with flat faces, and obsidian, black glassy
//! shards jutting from a low lump.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::blob::{Blob, cuts};
use super::lathe::Lathe;
use super::{Facet, span};
use crate::world::shapes::{FlatMesh, shade};

/// Triangles of a basalt rock: a hexagonal column and two pentagonal ones, six faces per corner each.
pub(super) const BASALT_TRIS: usize = 6 * (6 + 5 + 5);

/// Triangles of an obsidian rock: a low lump of 20 faces and four shards of four sides.
pub(super) const OBSIDIAN_TRIS: usize = 20 + 4 * 16;

/// Edge cut off the top of a basalt column, in metres at size one.
const CHAMFER: f32 = 0.05;

/// One column: corners, radius range, height range, distance from the middle and turn.
type Column = (usize, [f32; 2], [f32; 2], f32, f32);

/// The columns of a basalt rock: corners, radius and height ranges in metres, distance from the middle
/// in metres, and the share of a turn between it and the first one around.
const COLUMNS: [Column; 3] = [
    (6, [0.34, 0.42], [0.75, 1.0], 0.0, 0.0),
    (5, [0.26, 0.32], [0.45, 0.7], 0.62, 0.0),
    (5, [0.24, 0.3], [0.3, 0.5], 0.58, 0.36),
];

/// Paints a face of basalt column `column`: dark grey walls, a few of them weathered brown, a lighter edge
/// and a lighter top.
fn dark(f: &Facet, column: usize) -> [f32; 4] {
    let rust = (f.corner + column * 2).is_multiple_of(5);
    let rgb = match (f.band, rust) {
        (3, _) => [0.34, 0.34, 0.35],
        (2, _) => [0.28, 0.28, 0.3],
        (_, true) => [0.27, 0.23, 0.2],
        _ => [0.2, 0.2, 0.22],
    };
    shade(rgb, 0.9 + 0.05 * ((f.corner * 3 + column) % 4) as f32)
}

/// Adds a basalt rock of `size` drawn from `seed`: a tall hexagonal column with two shorter pentagonal
/// ones against it, each leaning a little and cut a little aslant on top, sunk a quarter metre.
pub(super) fn basalt(m: &mut FlatMesh, seed: u32, size: f32) {
    let around = span(seed, 1, [0.0, TAU]);
    for (k, (sides, radius, height, out, share)) in COLUMNS.into_iter().enumerate() {
        let s = seed.wrapping_add(101 * k as u32 + 7);
        let (r, h, c) = (
            span(s, 3, radius) * size,
            span(s, 4, height) * size,
            CHAMFER * size,
        );
        let a = around + share * TAU + span(s, 2, [-0.3, 0.3]);
        let lathe = Lathe {
            base: Vec3::new(a.cos(), 0.0, a.sin()) * out * size,
            sides,
            rings: vec![[-0.25 * size, r], [h - c, r], [h, r - c]],
            apex: h,
            lean: Vec2::new(span(s, 5, [-0.1, 0.1]), span(s, 6, [-0.1, 0.1])),
            tilt: Vec2::new(span(s, 7, [-0.12, 0.12]), span(s, 8, [-0.12, 0.12])),
            wobble: 0.12,
            seed: s,
        };
        lathe.add(m, &|f: &Facet| dark(f, k));
    }
}

/// Paints a face of obsidian: glassy black, with a cold grey sheen on the faces turned to the sky and a
/// faint violet glint on some edges.
fn glass(f: &Facet) -> [f32; 4] {
    let sheen = f.normal.y.max(0.0).powi(3);
    let edge = 0.04 * ((f.corner + f.band) % 3) as f32;
    let rgb = [
        0.05 + 0.24 * sheen + edge,
        0.05 + 0.22 * sheen + edge,
        0.07 + 0.3 * sheen + edge * 1.5,
    ];
    shade(rgb, 1.0)
}

/// Adds an obsidian rock drawn from `seed`: a low angular lump and four four-sided shards rising from
/// it, each leaning out its own way.
pub(super) fn obsidian(m: &mut FlatMesh, seed: u32) {
    let lump = Blob {
        centre: Vec3::Y * 0.04,
        radii: Vec3::new(0.42, 0.2, 0.38),
        level: 0,
        wobble: 0.2,
        seed,
        cuts: cuts(seed, 2, [0.5, 0.65], 0.6),
    };
    lump.add(m, &glass);
    let turn = span(seed, 1, [0.0, TAU]);
    for (k, tall) in [1.0, 0.7, 0.85, 0.6].into_iter().enumerate() {
        let s = seed.wrapping_add(71 * k as u32 + 5);
        let a = turn + k as f32 * TAU / 4.0 + span(s, 2, [-0.4, 0.4]);
        let out = Vec2::new(a.cos(), a.sin());
        let (r, h) = (span(s, 3, [0.1, 0.16]), span(s, 4, [0.5, 0.85]) * tall);
        let shard = Lathe {
            base: Vec3::new(out.x, 0.0, out.y) * span(s, 5, [0.04, 0.2]),
            sides: 4,
            rings: vec![[-0.12, r], [h * 0.6, r * 0.7]],
            apex: h,
            lean: out * span(s, 6, [0.2, 0.55]),
            tilt: Vec2::ZERO,
            wobble: 0.3,
            seed: s,
        };
        shard.add(m, &glass);
    }
}
