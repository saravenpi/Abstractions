//! Checks that every grown look is a closed solid within its triangle budget, buried where it is tall, and
//! that the book hands back the mesh of each look.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::life::flora::{Form, Paint};
use crate::world::propmesh::{BIG, merge_props};
use crate::world::scatter::Placement;
use crate::world::solids::tests::{triangles, volume};

/// Most triangles a grown plant of the big props has, as many as the weeping willow.
pub const BIG_TRIS: usize = 330;

/// Most triangles a grown plant of the ground cover has.
pub const SMALL_TRIS: usize = 120;

/// Returns the dials of architecture `arch` with every flag set to `on`, `layers` layers, and flowers,
/// fruit and leaves when `dressed`.
fn dials(arch: u8, on: bool, layers: u8, dressed: bool) -> Dials {
    let colour = |c: [f32; 3]| dressed.then_some(c);
    Dials {
        architecture: arch,
        form: Form {
            height: 3.0,
            layers,
            leaf: if on { 1.4 } else { 0.15 },
            needles: on && arch == 0 && layers.is_multiple_of(2),
            wax: on,
            spines: on,
            store: on,
            thorns: on,
            resin: on,
            deciduous: on,
        },
        paint: Paint {
            leaves: colour([0.05, 0.3, 0.04]),
            stem: [0.06, 0.035, 0.02],
            flowers: colour([0.85, 0.05, 0.25]),
            fruit: colour([0.9, 0.4, 0.02]),
        },
    }
}

/// Returns every look the tests sweep: each architecture, variant, flag setting, layer count and dressing.
fn looks() -> Vec<Look> {
    let flags = [false, true];
    let all = (0..8u8).flat_map(|a| (0..VARIANTS as u8).map(move |v| (a, v)));
    all.flat_map(|(a, v)| {
        flags
            .into_iter()
            .flat_map(move |on| (1..=4).map(move |l| (a, v, on, l)))
    })
    .flat_map(|(a, v, on, l)| {
        flags.map(|d| Look {
            dials: dials(a, on, l, d),
            variant: v,
        })
    })
    .collect()
}

/// Rounds a vertex to a grid so shared corners compare equal.
fn key(p: Vec3) -> [i32; 3] {
    (p * 1000.0).round().as_ivec3().to_array()
}

/// Every look is watertight or double sided, wound outward, and within the budget of its pass.
#[test]
fn every_look_is_closed_and_within_budget() {
    for look in looks() {
        let tris = triangles(&build(&look).into_mesh());
        let edges: HashSet<_> = tris
            .iter()
            .flat_map(|t| (0..3).map(|k| (key(t[k]), key(t[(k + 1) % 3]))))
            .collect();
        let open = edges
            .iter()
            .filter(|(a, b)| !edges.contains(&(*b, *a)))
            .count();
        assert_eq!(open, 0, "{look:?} has {open} open edges");
        assert!(volume(&tris) > -1e-4, "{look:?} is wound inside out");
        let big = (habit(look.dials.architecture).carrier as usize) < BIG;
        let budget = if big { BIG_TRIS } else { SMALL_TRIS };
        assert!(tris.len() <= budget, "{} triangles in {look:?}", tris.len());
    }
}

/// Trees, stalks and lattices start half a metre under the ground, like the trees of home.
#[test]
fn tall_looks_are_buried() {
    for look in looks()
        .into_iter()
        .filter(|l| matches!(l.dials.architecture, 0 | 5 | 7))
    {
        let low = triangles(&build(&look).into_mesh())
            .iter()
            .flatten()
            .map(|p| p.y)
            .fold(f32::MAX, f32::min);
        assert!(low < -0.35, "{look:?} starts at {low}");
    }
}

/// The book gives one variant past the shared meshes per look, and the merged mesh of a carrier with it is
/// the grown one.
#[test]
fn the_book_names_each_look() {
    let (a, b) = (
        Look {
            dials: dials(5, true, 3, true),
            variant: 1,
        },
        Look {
            dials: dials(1, false, 2, true),
            variant: 0,
        },
    );
    let (pa, pb) = (page(a), page(b));
    assert!(grown(pa) && grown(pb) && pa != pb);
    assert_eq!(page(a), pa);
    assert_eq!(tris(pa), build(&a).into_mesh().count_vertices() / 3);
    let placed = Placement {
        pos: Vec3::ZERO,
        yaw: 0.0,
        scale: 1.0,
        plant: habit(5).carrier,
        variant: pa,
    };
    assert_eq!(merge_props(&[placed]).count_vertices(), tris(pa) * 3);
}
