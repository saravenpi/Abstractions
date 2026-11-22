//! Checks the meshes of the stones: each kind has its own shape and colours, each variant its own
//! corners, every stone is closed, sunk into the ground and within its triangle budget.

use std::collections::HashSet;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::*;
use crate::world::solids::tests::{triangles, volume};

/// The rocks a player tells apart by their look: basalt, flint, obsidian, ice, sandstone and the six
/// planetary minerals.
const LOOKS: [Plant; 11] = [
    Plant::Rock,
    Plant::Flint,
    Plant::Obsidian,
    Plant::IceRock,
    Plant::Sandstone,
    Plant::Quartz,
    Plant::Jade,
    Plant::Ironstone,
    Plant::Malachite,
    Plant::Brimstone,
    Plant::Siderite,
];

/// Every stone, the basalt block of the deep floor included.
const ALL: [Plant; 12] = [
    Plant::Rock,
    Plant::Flint,
    Plant::Obsidian,
    Plant::IceRock,
    Plant::Sandstone,
    Plant::Boulder,
    Plant::Quartz,
    Plant::Jade,
    Plant::Ironstone,
    Plant::Malachite,
    Plant::Brimstone,
    Plant::Siderite,
];

/// Most triangles one stone may have.
const BUDGET: usize = 100;

/// Returns the colour lookups of a mesh, as bits.
fn palette(mesh: &Mesh) -> HashSet<[u32; 2]> {
    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
        return HashSet::new();
    };
    uvs.iter()
        .map(|uv| [uv[0].to_bits(), uv[1].to_bits()])
        .collect()
}

/// Returns the shape of a stone: its triangle count, its height over the ground and its reach from its
/// axis, in metres.
fn shape(mesh: &Mesh) -> [f32; 3] {
    let tris = triangles(mesh);
    let points = tris.iter().flatten();
    let top = points.clone().map(|p| p.y).fold(f32::MIN, f32::max);
    let reach = points.map(|p| p.xz().length()).fold(0.0, f32::max);
    [tris.len() as f32, top, reach]
}

/// Rounds a vertex to a grid so shared corners compare equal.
fn key(p: Vec3) -> [i32; 3] {
    (p * 1000.0).round().as_ivec3().to_array()
}

/// Returns how many edges of `tris` no other triangle runs back along: none for a closed solid.
fn open(tris: &[[Vec3; 3]]) -> usize {
    let edges: HashSet<_> = tris
        .iter()
        .flat_map(|t| (0..3).map(|k| (key(t[k]), key(t[(k + 1) % 3]))))
        .collect();
    let back = |(a, b): &&([i32; 3], [i32; 3])| !edges.contains(&(*b, *a));
    edges.iter().filter(back).count()
}

/// Every stone of every variant is watertight, wound outward, has no sliver of a triangle, sinks below
/// the ground, has as many triangles as the count says, and no more than the budget.
#[test]
fn every_stone_is_a_closed_sunk_solid_within_budget() {
    for plant in ALL {
        for variant in 0..VARIANTS {
            let tris = triangles(&big_mesh(plant as usize, variant));
            let gaps = open(&tris);
            assert_eq!(gaps, 0, "{plant:?} {variant} has {gaps} open edges");
            assert!(
                volume(&tris) > 0.01,
                "{plant:?} {variant} is wound inside out"
            );
            let thin = tris
                .iter()
                .filter(|[a, b, c]| (*b - *a).cross(*c - *a).length() < 1e-7);
            assert_eq!(thin.count(), 0, "{plant:?} {variant} has slivers");
            let low = tris.iter().flatten().map(|p| p.y).fold(f32::MAX, f32::min);
            assert!(
                low < -0.05,
                "{plant:?} {variant} sits on the ground at {low}"
            );
            assert_eq!(
                tris.len(),
                solid_tris(plant as usize),
                "{plant:?} {variant}"
            );
            assert!(
                tris.len() <= BUDGET,
                "{plant:?} has {} triangles",
                tris.len()
            );
        }
    }
}

/// No two kinds of rock look alike: their shapes differ by a triangle or by a tenth of a metre in height
/// or reach, and they share under half their colours.
#[test]
fn each_stone_has_its_own_shape_and_colours() {
    for (i, a) in LOOKS.iter().enumerate() {
        for b in &LOOKS[i + 1..] {
            let (ma, mb) = (big_mesh(*a as usize, 0), big_mesh(*b as usize, 0));
            let (sa, sb) = (shape(&ma), shape(&mb));
            let apart =
                sa[0] != sb[0] || (sa[1] - sb[1]).abs() > 0.1 || (sa[2] - sb[2]).abs() > 0.1;
            assert!(apart, "{a:?} {sa:?} and {b:?} {sb:?} have one shape");
            let (pa, pb) = (palette(&ma), palette(&mb));
            let shared = pa.intersection(&pb).count();
            assert!(
                shared * 2 < pa.len().min(pb.len()),
                "{a:?} and {b:?} share {shared} colours"
            );
        }
    }
}

/// The variants of a stone keep its triangles and move its corners: no two of them are one mesh.
#[test]
fn the_variants_of_a_stone_differ() {
    for plant in ALL {
        let corners: Vec<Vec<[Vec3; 3]>> = (0..VARIANTS)
            .map(|v| triangles(&big_mesh(plant as usize, v)))
            .collect();
        for (i, a) in corners.iter().enumerate() {
            for b in &corners[i + 1..] {
                assert_eq!(a.len(), b.len(), "{plant:?}");
                assert_ne!(a, b, "{plant:?} has two variants alike");
            }
        }
    }
}

/// Flint is the small stone, a nodule under half a metre high, and the basalt of the deep floor is
/// the big one, within the draft that keeps it under water.
#[test]
fn flint_is_small_and_the_boulder_big() {
    let [_, flint, _] = shape(&big_mesh(Plant::Flint as usize, 1));
    let [_, rock, _] = shape(&big_mesh(Plant::Rock as usize, 1));
    let [_, boulder, _] = shape(&big_mesh(Plant::Boulder as usize, 1));
    assert!(
        flint < 0.5 && flint < rock,
        "flint {flint} m, basalt {rock} m"
    );
    assert!(
        boulder > rock && boulder <= Plant::Boulder.draft(),
        "boulder {boulder} m"
    );
}
