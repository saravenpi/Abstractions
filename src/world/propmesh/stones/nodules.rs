//! The rounded stones: flint nodules with dark flaked facets, angular blocks of ice and layered
//! sandstone boulders, each a large blob with a small one beside it.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::blob::{Blob, cuts};
use super::{Facet, span};
use crate::world::shapes::{FlatMesh, shade};

/// Triangles of a flint rock: a nodule of 80 faces and a small one of 20.
pub(super) const FLINT_TRIS: usize = 80 + 20;

/// Triangles of an ice rock: a block of 80 faces and a chip of 20.
pub(super) const ICE_TRIS: usize = 80 + 20;

/// Triangles of a sandstone rock: a boulder of 80 faces and a small one of 20.
pub(super) const SANDSTONE_TRIS: usize = 80 + 20;

/// Height of one bed of sandstone, in metres.
const BED: f32 = 0.16;

/// One blob of a clump: how often its icosahedron is split, its radii and its middle in metres, and how
/// many cuts it has.
type Lump = (u32, Vec3, Vec3, usize);

/// Adds the blobs of `lumps` to `m`, each with a seed of its own drawn from `seed`, moved by `wobble` and
/// cut between the two ends of `depth` from its middle by cuts rising at most `rise`, painted by `paint`.
fn clump(
    m: &mut FlatMesh,
    lumps: [Lump; 2],
    seed: u32,
    (wobble, depth, rise): (f32, [f32; 2], f32),
    paint: &dyn Fn(&Facet) -> [f32; 4],
) {
    for (k, (level, radii, centre, count)) in lumps.into_iter().enumerate() {
        let s = seed.wrapping_add(53 * k as u32 + 3);
        let blob = Blob {
            centre,
            radii,
            level,
            wobble,
            seed: s,
            cuts: cuts(s, count, depth, rise),
        };
        blob.add(m, paint);
    }
}

/// Returns the middle of the small blob of a clump: `out` metres from the middle at a turn drawn from
/// `seed`, `up` metres high.
fn beside(seed: u32, out: f32, up: f32) -> Vec3 {
    let a = span(seed, 2, [0.0, TAU]);
    Vec3::new(a.cos() * out, up, a.sin() * out)
}

/// Paints a face of flint: the pale grey-beige rind, or a dark grey-brown facet where a piece flaked off,
/// a little lighter where it faces the sky.
fn flake(f: &Facet) -> [f32; 4] {
    let glint = 0.9 + 0.25 * f.normal.y.max(0.0);
    match f.band {
        0 => shade([0.64, 0.6, 0.52], 0.93 + 0.04 * (f.corner % 3) as f32),
        b if b % 2 == 1 => shade([0.3, 0.27, 0.23], glint),
        _ => shade([0.23, 0.21, 0.18], glint),
    }
}

/// Adds a flint rock drawn from `seed`: a knobbly nodule a little over half a metre across with three
/// flaked facets, and a small nodule beside it.
pub(super) fn flint(m: &mut FlatMesh, seed: u32) {
    let size = span(seed, 1, [0.9, 1.1]);
    let lumps = [
        (1, Vec3::new(0.32, 0.26, 0.28) * size, Vec3::Y * 0.17, 3),
        (0, Vec3::new(0.15, 0.12, 0.14), beside(seed, 0.36, 0.06), 1),
    ];
    clump(m, lumps, seed, (0.14, [0.55, 0.72], 0.8), &flake);
}

/// Paints a face of ice: frost white where it faces the sky, pale cyan on its facets and a deep blue in
/// the rest, as if the light went into it.
fn frost(f: &Facet) -> [f32; 4] {
    let rgb = match (f.band, f.normal.y > 0.6) {
        (_, true) => [0.9, 0.95, 1.0],
        (0, false) => [0.42, 0.62, 0.84],
        (b, false) if b % 2 == 1 => [0.66, 0.83, 0.95],
        _ => [0.54, 0.74, 0.9],
    };
    shade(rgb, 0.96 + 0.04 * (f.corner % 2) as f32)
}

/// Adds an ice rock drawn from `seed`: an angular block about a metre high, cut flat on six sides, and a
/// chip of ice beside it.
pub(super) fn ice(m: &mut FlatMesh, seed: u32) {
    let size = span(seed, 1, [0.9, 1.1]);
    let lumps = [
        (1, Vec3::new(0.62, 0.55, 0.56) * size, Vec3::Y * 0.36, 6),
        (0, Vec3::new(0.26, 0.22, 0.24), beside(seed, 0.62, 0.1), 2),
    ];
    clump(m, lumps, seed, (0.04, [0.62, 0.8], 1.0), &frost);
}

/// Paints a face of sandstone by the bed it lies in: orange, cream and rust layers, lighter where it is
/// cut flat.
fn layer(f: &Facet) -> [f32; 4] {
    let bed = ((f.centre.y + 1.0) / BED) as usize % 3;
    let rgb = [[0.8, 0.57, 0.35], [0.88, 0.73, 0.53], [0.7, 0.45, 0.28]][bed];
    let tone = match f.band {
        0 => 0.95 + 0.03 * (f.corner % 3) as f32,
        _ => 1.06,
    };
    shade(rgb, tone)
}

/// Adds a sandstone rock drawn from `seed`: a broad rounded boulder with a flat top and one flat side,
/// banded in beds, and a small boulder beside it.
pub(super) fn sandstone(m: &mut FlatMesh, seed: u32) {
    let size = span(seed, 1, [0.9, 1.1]);
    let mut flat = cuts(seed, 1, [0.68, 0.78], 0.1);
    flat.push((Vec3::Y, span(seed, 3, [0.55, 0.7])));
    let boulder = Blob {
        centre: Vec3::Y * 0.4,
        radii: Vec3::new(0.95, 0.62, 0.82) * size,
        level: 1,
        wobble: 0.07,
        seed,
        cuts: flat,
    };
    boulder.add(m, &layer);
    let small = seed ^ 0x5A5A;
    let pebble = Blob {
        centre: beside(seed, 0.92, 0.12),
        radii: Vec3::new(0.4, 0.28, 0.36),
        level: 0,
        wobble: 0.1,
        seed: small,
        cuts: cuts(small, 1, [0.6, 0.7], 0.6),
    };
    pebble.add(m, &layer);
}
