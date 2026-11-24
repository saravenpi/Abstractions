//! The stones: each kind of rock prop has a builder of its own, with its own silhouette and colours, so
//! the stone a rock is made of reads from 8 m. A builder makes the same number of triangles for each of
//! its variants; the variant only moves the corners, the sizes and the cuts.

mod blob;
mod columns;
mod lathe;
mod minerals;
mod nodules;

use bevy::prelude::*;

use super::Plant;
use crate::world::shapes::FlatMesh;

/// The rock props that have a builder of their own.
pub(super) const STONES: [Plant; 12] = [
    Plant::Rock,
    Plant::IceRock,
    Plant::Sandstone,
    Plant::Boulder,
    Plant::Flint,
    Plant::Obsidian,
    Plant::Quartz,
    Plant::Jade,
    Plant::Ironstone,
    Plant::Malachite,
    Plant::Brimstone,
    Plant::Siderite,
];

/// Size of the basalt block of the deep floor against the basalt of the land, unitless: it stays under
/// the draft of the boulder.
const BOULDER: f32 = 1.7;

/// What a builder knows of one face when it paints it.
struct Facet {
    /// Unit normal of the face, in the frame of the prop.
    normal: Vec3,
    /// Middle of the face, in metres in the frame of the prop.
    centre: Vec3,
    /// The part of its solid the face belongs to. On a blob: 0 on the rounded rind, else one more than the
    /// cut it lies on. On a lathe: 0 on the bottom, then the walls ring by ring, then the top.
    band: usize,
    /// The corner the face starts at around its ring, or the number of the face in its blob.
    corner: usize,
}

/// Returns a stable value in `[0, 1)` for `seed` and `salt`.
fn hash(seed: u32, salt: u32) -> f32 {
    let h = (seed ^ salt.wrapping_mul(0x9E37_79B9)).wrapping_mul(0x85EB_CA6B);
    let h = (h ^ (h >> 13)).wrapping_mul(0xC2B2_AE35);
    ((h ^ (h >> 16)) >> 8) as f32 / 16_777_216.0
}

/// Returns a stable value between the two ends of `range` for `seed` and `salt`.
fn span(seed: u32, salt: u32, range: [f32; 2]) -> f32 {
    range[0] + (range[1] - range[0]) * hash(seed, salt)
}

/// Adds the triangle `corners` to `m`, painted by `paint` as part `band` from corner `corner` of its solid.
fn lay(
    m: &mut FlatMesh,
    corners: [Vec3; 3],
    (band, corner): (usize, usize),
    paint: &dyn Fn(&Facet) -> [f32; 4],
) {
    let [a, b, c] = corners;
    let facet = Facet {
        normal: (b - a).cross(c - a).normalize_or_zero(),
        centre: (a + b + c) / 3.0,
        band,
        corner,
    };
    m.tri(a, b, c, paint(&facet));
}

/// Returns the stone that solid plant number `kind` names, if it is one.
fn stone(kind: usize) -> Option<Plant> {
    STONES.into_iter().find(|p| *p as usize == kind)
}

/// Returns the index in [`STONES`] of plant `plant`, or nothing when `plant` is not a stone.
pub(super) fn stone_index(plant: Plant) -> Option<usize> {
    STONES.iter().position(|p| *p == plant)
}

/// Returns the mesh of stone `kind`, an index into [`Plant`], variant `variant`, its base at the origin, or
/// nothing when the kind is no stone.
pub(super) fn mesh(kind: usize, variant: usize) -> Option<FlatMesh> {
    let plant = stone(kind)?;
    let seed = (kind as u32).wrapping_mul(977) ^ (variant as u32 + 1).wrapping_mul(0x2545_F491);
    let mut m = FlatMesh::default();
    match plant {
        Plant::Rock => columns::basalt(&mut m, seed, 1.0),
        Plant::Boulder => columns::basalt(&mut m, seed, BOULDER),
        Plant::Obsidian => columns::obsidian(&mut m, seed),
        Plant::Flint => nodules::flint(&mut m, seed),
        Plant::IceRock => nodules::ice(&mut m, seed),
        Plant::Quartz => minerals::quartz(&mut m, seed),
        Plant::Jade => minerals::jade(&mut m, seed),
        Plant::Ironstone => minerals::ironstone(&mut m, seed),
        Plant::Malachite => minerals::malachite(&mut m, seed),
        Plant::Brimstone => minerals::brimstone(&mut m, seed),
        Plant::Siderite => minerals::siderite(&mut m, seed),
        _ => nodules::sandstone(&mut m, seed),
    }
    Some(m)
}

/// Returns how many triangles stone `kind` has, the same for every variant, or nothing when the kind is no
/// stone.
pub(super) fn tris(kind: usize) -> Option<usize> {
    let count = match stone(kind)? {
        Plant::Rock | Plant::Boulder => columns::BASALT_TRIS,
        Plant::Obsidian => columns::OBSIDIAN_TRIS,
        Plant::Flint => nodules::FLINT_TRIS,
        Plant::IceRock => nodules::ICE_TRIS,
        Plant::Quartz => minerals::QUARTZ_TRIS,
        Plant::Jade => minerals::JADE_TRIS,
        Plant::Ironstone => minerals::IRONSTONE_TRIS,
        Plant::Malachite => minerals::MALACHITE_TRIS,
        Plant::Brimstone => minerals::BRIMSTONE_TRIS,
        Plant::Siderite => minerals::SIDERITE_TRIS,
        _ => nodules::SANDSTONE_TRIS,
    };
    Some(count)
}
