//! The six planetary minerals: quartz crystal prisms, emerald jade boulders, banded ironstone
//! columns, copper-green malachite nodules, sulfur-yellow brimstone shards and nickel-iron siderite
//! meteorites.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::blob::{Blob, cuts};
use super::lathe::Lathe;
use super::{Facet, span};
use crate::world::shapes::{FlatMesh, shade};

/// Triangles of a quartz rock: a base lump of 20 faces and three hexagonal prisms of 24 faces.
pub(super) const QUARTZ_TRIS: usize = 20 + 3 * 24;

/// Triangles of a jade boulder: one faceted 80-face nephrite stone.
pub(super) const JADE_TRIS: usize = 80;

/// Triangles of an ironstone rock: banded columns of 5, 4 and 4 sides with three rings each.
pub(super) const IRONSTONE_TRIS: usize = 6 * (5 + 4 + 4);

/// Triangles of a malachite rock: three clustered nodules of 20 faces each.
pub(super) const MALACHITE_TRIS: usize = 3 * 20;

/// Triangles of a brimstone rock: a base lump of 20 faces and three four-sided shards of 16.
pub(super) const BRIMSTONE_TRIS: usize = 20 + 3 * 16;

/// Triangles of a siderite meteorite: an eight-sided core of 32 faces and two 20-face lobes.
pub(super) const SIDERITE_TRIS: usize = 32 + 2 * 20;

/// Adds a base crust blob at `centre` with `radii` and `wobble` to `m`.
fn crust(
    m: &mut FlatMesh,
    (centre, radii): (Vec3, Vec3),
    wobble: f32,
    seed: u32,
    paint: &dyn Fn(&Facet) -> [f32; 4],
) {
    let blob = Blob {
        centre,
        radii,
        level: 0,
        wobble,
        seed,
        cuts: cuts(seed, 2, [0.55, 0.72], 0.65),
    };
    blob.add(m, paint);
}

/// Adds three tapering spires of `sides` faces around `m` with `talls`, `taper` and `wobble`.
fn spires(
    m: &mut FlatMesh,
    seed: u32,
    (sides, talls, taper, wobble, lean): (usize, [f32; 3], [f32; 2], f32, [f32; 2]),
    paint: &dyn Fn(&Facet) -> [f32; 4],
) {
    let turn = span(seed, 1, [0.0, TAU]);
    for (k, tall) in talls.into_iter().enumerate() {
        let s = seed.wrapping_add(67 * k as u32 + 11);
        let a = turn + k as f32 * TAU / 3.0 + span(s, 2, [-0.35, 0.35]);
        let out = Vec2::new(a.cos(), a.sin());
        let (r, h) = (span(s, 3, [0.11, 0.16]), span(s, 4, [0.5, 0.85]) * tall);
        let spire = Lathe {
            base: Vec3::new(out.x, 0.0, out.y) * span(s, 5, [0.04, 0.2]),
            sides,
            rings: vec![[-0.14, r], [h * taper[0], r * taper[1]]],
            apex: h,
            lean: out * span(s, 6, lean),
            tilt: Vec2::ZERO,
            wobble,
            seed: s,
        };
        spire.add(m, paint);
    }
}

/// Adds a quartz rock drawn from `seed`: three tilted pale rose-white hexagonal crystal prisms
/// rising from a rough silica lump.
pub(super) fn quartz(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let base = match (f.band >= 2, f.corner.is_multiple_of(2)) {
            (true, _) => [0.96, 0.92, 0.93],
            (false, true) => [0.90, 0.80, 0.83],
            (false, false) => [0.82, 0.70, 0.75],
        };
        shade(base, 0.94 + 0.04 * (f.corner % 3) as f32)
    };
    crust(
        m,
        (Vec3::Y * 0.03, Vec3::new(0.36, 0.18, 0.34)),
        0.16,
        seed,
        &paint,
    );
    spires(
        m,
        seed,
        (6, [1.0, 0.74, 0.58], [0.74, 0.9], 0.08, [0.08, 0.32]),
        &paint,
    );
}

/// Adds a jade boulder drawn from `seed`: a dense emerald nephrite stone with four smooth facets.
pub(super) fn jade(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let glint = 0.92 + 0.18 * f.normal.y.max(0.0);
        match f.band {
            0 => shade([0.16, 0.46, 0.26], 0.94 + 0.04 * (f.corner % 3) as f32),
            b if b % 2 == 1 => shade([0.24, 0.58, 0.34], glint),
            _ => shade([0.12, 0.38, 0.22], glint),
        }
    };
    let size = span(seed, 1, [0.92, 1.1]);
    let boulder = Blob {
        centre: Vec3::Y * 0.18,
        radii: Vec3::new(0.48, 0.36, 0.42) * size,
        level: 1,
        wobble: 0.08,
        seed,
        cuts: cuts(seed, 4, [0.60, 0.76], 0.85),
    };
    boulder.add(m, &paint);
}

/// Adds an ironstone rock drawn from `seed`: three stepped dark hematite and rust-banded columns.
pub(super) fn ironstone(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let layer = ((f.centre.y + 1.0) / 0.15) as usize % 3;
        let rgb = [[0.46, 0.16, 0.12], [0.58, 0.22, 0.14], [0.28, 0.13, 0.12]][layer];
        shade(rgb, 0.92 + 0.06 * ((f.band + f.corner) % 3) as f32)
    };
    let around = span(seed, 1, [0.0, TAU]);
    let specs = [
        (5, [0.30, 0.38], [0.68, 0.92], 0.0, 0.0),
        (4, [0.24, 0.30], [0.42, 0.64], 0.54, 0.08),
        (4, [0.22, 0.28], [0.28, 0.46], 0.50, 0.44),
    ];
    for (k, (sides, radius, height, out, share)) in specs.into_iter().enumerate() {
        let s = seed.wrapping_add(89 * k as u32 + 5);
        let (r, h) = (span(s, 3, radius), span(s, 4, height));
        let a = around + share * TAU + span(s, 2, [-0.25, 0.25]);
        let col = Lathe {
            base: Vec3::new(a.cos(), 0.0, a.sin()) * out,
            sides,
            rings: vec![[-0.22, r], [h - 0.04, r], [h, r - 0.04]],
            apex: h,
            lean: Vec2::new(span(s, 5, [-0.08, 0.08]), span(s, 6, [-0.08, 0.08])),
            tilt: Vec2::new(span(s, 7, [-0.14, 0.14]), span(s, 8, [-0.14, 0.14])),
            wobble: 0.14,
            seed: s,
        };
        col.add(m, &paint);
    }
}

/// Adds a malachite rock drawn from `seed`: three clustered copper-green banded carbonate nodules.
pub(super) fn malachite(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let ring = ((f.centre.y + 1.0) / 0.11) as usize % 3;
        let rgb = [[0.08, 0.52, 0.40], [0.16, 0.68, 0.52], [0.06, 0.36, 0.28]][ring];
        shade(rgb, 0.92 + 0.06 * ((f.band + f.corner) % 2) as f32)
    };
    let (size, turn) = (span(seed, 1, [0.92, 1.1]), span(seed, 2, [0.0, TAU]));
    let lumps = [
        (Vec3::Y * 0.14, Vec3::new(0.36, 0.28, 0.34) * size),
        (
            Vec3::new(turn.cos() * 0.36, 0.09, turn.sin() * 0.36),
            Vec3::new(0.22, 0.18, 0.20),
        ),
        (
            Vec3::new((turn + 2.2).cos() * 0.32, 0.07, (turn + 2.2).sin() * 0.32),
            Vec3::new(0.18, 0.15, 0.17),
        ),
    ];
    for (k, spec) in lumps.into_iter().enumerate() {
        crust(m, spec, 0.12, seed.wrapping_add(47 * k as u32 + 7), &paint);
    }
}

/// Adds a brimstone rock drawn from `seed`: jagged sulfur-yellow crystal shards on a low crust.
pub(super) fn brimstone(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let up = f.normal.y.max(0.0);
        let tint = 0.10 + 0.10 * ((f.corner + f.band) % 3) as f32;
        shade(
            [0.84 + 0.14 * up, 0.72 + 0.18 * up, tint],
            0.94 + 0.04 * (f.corner % 3) as f32,
        )
    };
    crust(
        m,
        (Vec3::Y * 0.03, Vec3::new(0.38, 0.18, 0.35)),
        0.2,
        seed,
        &paint,
    );
    spires(
        m,
        seed,
        (4, [1.0, 0.78, 0.62], [0.56, 0.72], 0.26, [0.18, 0.48]),
        &paint,
    );
}

/// Adds a siderite meteorite drawn from `seed`: a pitted dark metallic nickel-iron core with two
/// fused lobes.
pub(super) fn siderite(m: &mut FlatMesh, seed: u32) {
    let paint = |f: &Facet| {
        let top = [0.36, 0.38, 0.44].map(|c| c + 0.1 * f.normal.y.max(0.0));
        let rgb = [top, [0.46, 0.42, 0.36], [0.52, 0.54, 0.6]][f.band.min(2)];
        shade(rgb, 0.94 + 0.05 * (f.corner % 3) as f32)
    };
    let (r0, r1) = (span(seed, 3, [0.32, 0.4]), span(seed, 5, [0.24, 0.3]));
    let lean = Vec2::new(span(seed, 7, [-0.14, 0.14]), span(seed, 8, [-0.14, 0.14]));
    Lathe {
        base: Vec3::ZERO,
        sides: 8,
        rings: vec![[-0.16, r0], [span(seed, 4, [0.34, 0.46]), r1]],
        apex: span(seed, 6, [0.48, 0.64]),
        lean,
        tilt: Vec2::ZERO,
        wobble: 0.22,
        seed,
    }
    .add(m, &paint);
    let turn = span(seed, 1, [0.0, TAU]);
    for (k, scale) in [1.0, 0.82].into_iter().enumerate() {
        let a = turn + k as f32 * 2.7;
        let spot = (
            Vec3::new(a.cos() * 0.25, 0.08, a.sin() * 0.25),
            Vec3::new(0.24, 0.2, 0.22) * scale,
        );
        crust(m, spot, 0.18, seed.wrapping_add(59 * k as u32 + 13), &paint);
    }
}
