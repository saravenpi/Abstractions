//! Tests of the embedding and of the baked planet: chart geometry lands on the sphere.

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::prelude::*;

use super::embed::Embedding;
use super::tests::view;
use crate::world::Globe;
use crate::world::globe::Face;
use crate::world::globe::far::{CELLS, Range, bake, nodes, range};
use crate::world::shapes::FlatMesh;

/// Returns a grid of flat triangles over a whole chunk in chunk space, `y` metres above the sea.
fn flat_grid(y: f32) -> Mesh {
    let mut mesh = FlatMesh::default();
    let at = |i: i32, j: i32| Vec3::new(i as f32 * 8.0 - 32.0, y, j as f32 * 8.0 - 32.0);
    for (i, j) in (0..8).flat_map(|i| (0..8).map(move |j| (i, j))) {
        mesh.tri(at(i, j), at(i, j + 1), at(i + 1, j), [1.0; 4]);
        mesh.tri(at(i + 1, j), at(i, j + 1), at(i + 1, j + 1), [1.0; 4]);
    }
    mesh.into_mesh()
}

/// Returns the float triples of attribute `id` of `mesh`.
fn triples(mesh: &Mesh, id: bevy::mesh::MeshVertexAttribute) -> Vec<Vec3> {
    let values = mesh.attribute(id).and_then(|a| a.as_float3());
    values
        .expect("a float3 attribute")
        .iter()
        .map(|p| Vec3::from(*p))
        .collect()
}

/// Embedding a flat grid puts every vertex at radius plus height from the planet centre within
/// 1 mm, with its normal along the local up, on small and large globes and at a cube corner.
#[test]
fn embedded_flat_grid_sits_on_the_sphere() {
    for (radius, face, corner) in [
        (3000.0, Face::PosX, false),
        (6000.0, Face::NegZ, true),
        (40_000.0, Face::PosY, true),
    ] {
        let terrain = view(radius, face);
        let edge = (1i32 << terrain.globe().leaf_level()) / 2 - 1;
        let coord = if corner {
            IVec2::splat(edge)
        } else {
            IVec2::new(2, -7)
        };
        let warp = Embedding::of(&terrain, coord);
        let mut mesh = flat_grid(12.5);
        warp.mesh(&mut mesh);
        let frame = warp.frame();
        let positions = triples(&mesh, Mesh::ATTRIBUTE_POSITION);
        let normals = triples(&mesh, Mesh::ATTRIBUTE_NORMAL);
        assert_eq!(positions.len(), 8 * 8 * 6);
        for (p, n) in positions.iter().zip(&normals) {
            let pos = frame.origin + frame.rot * p.as_dvec3();
            assert!((pos.length() - (radius + 12.5)).abs() < 1e-3);
            let up = frame.turn(pos.normalize());
            assert!(n.dot(up) > 0.9999, "normal {n} against up {up}");
        }
    }
}

/// A compound collider is embedded part by part: each part lands where a point does and stands
/// along the local up.
#[test]
fn embedded_colliders_follow_the_ground() {
    let terrain = view(3000.0, Face::NegY);
    let warp = Embedding::of(&terrain, IVec2::new(-20, 11));
    let at = Vec3::new(30.0, 4.0, -29.0);
    let trunk = Collider::compound(vec![(at, Quat::IDENTITY, Collider::cylinder(0.3, 2.0))]);
    let moved = warp.collider(&trunk);
    let parts = moved
        .shape()
        .as_compound()
        .expect("still a compound")
        .shapes();
    let (t, r) = (parts[0].0.translation, parts[0].0.rotation);
    let pos = Vec3::new(t.x, t.y, t.z);
    assert!((pos - warp.point(at)).length() < 1e-5);
    let axis = Quat::from_xyzw(r.x, r.y, r.z, r.w) * Vec3::Y;
    let frame = warp.frame();
    let up = frame.turn((frame.origin + frame.rot * pos.as_dvec3()).normalize());
    assert!(axis.dot(up) > 0.999_999);
    assert!(axis.dot(Vec3::Y) < 0.999_999);
}

/// The baked planet replaces the quadtree beyond 1.3 radii, stays down to 1.2 and stops at the
/// hand-over of 1.5, and its mesh holds sixteen nodes per face.
#[test]
fn far_planet_has_hysteresis_and_ninety_six_nodes() {
    assert_eq!(range(1.25, false), Range::Near);
    assert_eq!(range(1.25, true), Range::Far);
    assert_eq!(range(1.35, false), Range::Far);
    assert_eq!(range(1.15, true), Range::Near);
    assert_eq!(range(1.55, true), Range::Beyond);
    assert_eq!(nodes().len(), 96);
    let globe = Arc::new(Globe::new(9, 3000.0, 22.0));
    let mesh = bake(&globe);
    assert_eq!(mesh.count_vertices(), 96 * CELLS * CELLS * 6);
    let radii = triples(&mesh, Mesh::ATTRIBUTE_POSITION);
    assert!(
        radii
            .iter()
            .all(|p| p.length() > 2998.0 && p.length() < 3400.0)
    );
}
