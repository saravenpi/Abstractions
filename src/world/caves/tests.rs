//! Unit tests for deterministic cave planning, walkable slopes, ceiling clearance, portal hole
//! carving, and merged cave shell geometry.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::mesh::build_cave_shell;
use super::plan::{CavePlan, build_plan, portal_in_cell};
use crate::world::mesh::{GRID_POINTS, Lod, chunk_heights, chunk_mesh};
use crate::world::{CHUNK_SIZE, Terrain};

/// Finds a deterministic cave plan on `terrain` by scanning nearby 320 m cells.
fn first_cave(terrain: &Terrain) -> CavePlan {
    for cx in -6..=6 {
        for cz in -6..=6 {
            if let Some(portal) = portal_in_cell(terrain, IVec2::new(cx, cz)) {
                return build_plan(terrain, portal);
            }
        }
    }
    panic!("expected at least one cave within scanned dry land cells");
}

/// Same terrain seed produces identical cave portal, biome, spine, and alcove geometry.
#[test]
fn deterministic_generation() {
    let a = first_cave(&Terrain::new(42));
    let b = first_cave(&Terrain::new(42));
    assert_eq!(a, b);
    assert!(a.entry_radius >= 3.6 && a.entry_radius <= 5.2);
    assert!(!a.alcoves.is_empty());
}

/// Every tunnel and alcove segment stays within walkable slope limits (<= 28 degrees).
#[test]
fn walkable_slopes_under_limit() {
    let plan = first_cave(&Terrain::new(42));
    let max_rad = 28.0_f32.to_radians();
    let check = |spine: &[super::plan::SpineNode]| {
        for pair in spine.windows(2) {
            let delta = pair[1].pos - pair[0].pos;
            let horiz = Vec2::new(delta.x, delta.z).length().max(1e-4);
            let angle = (delta.y.abs() / horiz).atan();
            assert!(
                angle <= max_rad,
                "slope {:.1} deg exceeded 28 deg",
                angle.to_degrees()
            );
        }
    };
    check(&plan.main_spine);
    for alcove in &plan.alcoves {
        check(&alcove.spine);
    }
}

/// Cave ceilings past the entrance collar stay strictly below surface height minus 2.5 m.
#[test]
fn ceiling_clearance_below_surface() {
    let terrain = Terrain::new(42);
    let plan = first_cave(&terrain);
    for node in plan.main_spine.iter().skip(2) {
        let surf = terrain.height(plan.entry_xz.x + node.pos.x, plan.entry_xz.y + node.pos.z);
        let ceil = node.pos.y + node.ry;
        assert!(
            ceil < surf - 2.49,
            "ceiling {ceil} too close to surface {surf}"
        );
    }
}

/// Surface portal carving removes triangles inside the cave mouth on `Lod::Near` only.
#[test]
fn portal_hole_removes_surface_triangles() {
    let terrain = Terrain::new(42);
    let plan = first_cave(&terrain);
    let coord = (plan.entry_xz / CHUNK_SIZE).floor().as_ivec2();
    let full = chunk_heights(&terrain, coord, 1);
    let near = chunk_mesh(&terrain, coord, &full, Lod::Near);
    let n = GRID_POINTS - 1;
    let full_verts = (n * n * 2 + n * 8) * 3;
    assert!(
        near.count_vertices() < full_verts,
        "expected portal hole to remove triangles: {} vs {}",
        near.count_vertices(),
        full_verts
    );
    let mid_h = chunk_heights(&terrain, coord, Lod::Mid.stride());
    let mid = chunk_mesh(&terrain, coord, &mid_h, Lod::Mid);
    let nm = n / Lod::Mid.stride();
    assert_eq!(mid.count_vertices(), (nm * nm * 2 + nm * 8) * 3);
}

/// Merged cave shell mesh has finite vertices, unit normals, and inward-facing tunnel walls.
#[test]
fn merged_cave_shell_has_finite_vertices_and_inward_normals() {
    let plan = first_cave(&Terrain::new(42));
    let (mesh, _collider) = build_cave_shell(&plan);
    let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("missing positions");
    };
    let Some(VertexAttributeValues::Float32x3(norm)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        panic!("missing normals");
    };
    assert!(pos.len() >= 900, "too few shell vertices: {}", pos.len());
    assert!(pos.iter().all(|p| p.iter().all(|v| v.is_finite())));
    assert!(norm.iter().all(|n| Vec3::from(*n).length() > 0.9));
    let mut inward = 0usize;
    let main_tris = (plan.main_spine.len() - 1) * 14 * 2;
    for t in 0..main_tris {
        let base = t * 3;
        let center =
            (Vec3::from(pos[base]) + Vec3::from(pos[base + 1]) + Vec3::from(pos[base + 2])) / 3.0;
        let spine_k = (t / 28).min(plan.main_spine.len() - 2);
        let seg_mid = (plan.main_spine[spine_k].pos + plan.main_spine[spine_k + 1].pos) * 0.5;
        if Vec3::from(norm[base]).dot(seg_mid - center) > 0.0 {
            inward += 1;
        }
    }
    assert!(
        inward * 10 >= main_tris * 9,
        "expected inward normals: {inward}/{main_tris}"
    );
}
