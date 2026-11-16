//! Checks that every part of every solid plant is a closed outward solid or a double sided sheet.

use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::world::solids::tests::{triangles, volume};

/// Rounds a vertex to a grid so shared corners compare equal.
fn key(p: Vec3) -> [i32; 3] {
    [
        (p.x * 1000.0).round() as i32,
        (p.y * 1000.0).round() as i32,
        (p.z * 1000.0).round() as i32,
    ]
}

/// Every part is watertight with consistent winding and encloses a positive or zero volume.
#[test]
fn every_part_is_closed_or_double_sided() {
    for kind in (0..BIG).chain(PARTS) {
        for variant in 0..VARIANTS {
            for i in 0..spec(kind).len() {
                let mut m = FlatMesh::default();
                add_part(&mut m, kind, variant, i);
                let tris = triangles(&m.into_mesh());
                let edges: HashSet<_> = tris
                    .iter()
                    .flat_map(|t| (0..3).map(|k| (key(t[k]), key(t[(k + 1) % 3]))))
                    .collect();
                let open = edges
                    .iter()
                    .filter(|(a, b)| !edges.contains(&(*b, *a)))
                    .count();
                assert_eq!(open, 0, "kind {kind} part {i} has {open} open edges");
                assert!(
                    volume(&tris) > -1e-4,
                    "kind {kind} part {i} is wound inside out"
                );
            }
        }
    }
}

/// The counted triangles match the built meshes.
#[test]
fn tri_counts_match() {
    for kind in (0..BIG).chain(PARTS) {
        assert_eq!(
            big_mesh(kind, 1).count_vertices(),
            solid_tris(kind) * 3,
            "kind {kind}"
        );
    }
}
