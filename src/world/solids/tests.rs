//! Checks that every solid helper emits closed, outward facing meshes and that sheets are double sided.

use bevy::mesh::VertexAttributeValues;

use super::*;
use crate::world::shapes::{FlatMesh, srgb};

/// Returns the triangles of a mesh built from [`FlatMesh`].
pub(crate) fn triangles(mesh: &Mesh) -> Vec<[Vec3; 3]> {
    let Some(VertexAttributeValues::Float32x3(v)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        return Vec::new();
    };
    v.chunks(3)
        .map(|t| [Vec3::from(t[0]), Vec3::from(t[1]), Vec3::from(t[2])])
        .collect()
}

/// Returns the signed volume enclosed by a triangle soup.
pub(crate) fn volume(tris: &[[Vec3; 3]]) -> f32 {
    tris.iter().map(|t| t[0].dot(t[1].cross(t[2]))).sum::<f32>() / 6.0
}

/// Builds a mesh with `add` and returns its triangles.
fn build(add: impl Fn(&mut FlatMesh)) -> Vec<[Vec3; 3]> {
    let mut m = FlatMesh::default();
    add(&mut m);
    triangles(&m.into_mesh())
}

/// Returns one closed convex mesh per primitive, with its name.
fn closed_meshes() -> Vec<(&'static str, Vec<[Vec3; 3]>)> {
    let c = srgb(0.5, 0.5, 0.5);
    let base = Vec3::new(0.3, 1.0, -0.4);
    let tip = base + Vec3::new(0.5, 2.0, 0.3);
    vec![
        (
            "prism",
            build(|m| prism(m, base, Vec2::new(1.0, 2.0), 7, c)),
        ),
        (
            "frustum",
            build(|m| frustum(m, base, tip, Vec2::new(0.5, 0.2), c)),
        ),
        (
            "frustum down",
            build(|m| frustum(m, base, base - Vec3::Y * 2.0, Vec2::splat(0.3), c)),
        ),
        ("cone", build(|m| cone(m, base, Vec2::new(1.0, 2.0), 7, c))),
        (
            "icosa",
            build(|m| icosa(m, base, Vec3::new(1.0, 0.5, 0.8), (0.0, 3), c)),
        ),
    ]
}

/// Closed convex primitives have a positive volume and every face points away from the centroid.
#[test]
fn closed_primitives_face_outward() {
    for (name, tris) in closed_meshes() {
        let mid = tris.iter().flatten().sum::<Vec3>() / (tris.len() * 3) as f32;
        assert!(volume(&tris) > 0.0, "{name} volume {}", volume(&tris));
        for t in &tris {
            let normal = (t[1] - t[0]).cross(t[2] - t[0]);
            let center = (t[0] + t[1] + t[2]) / 3.0;
            assert!(
                normal.dot(center - mid) > 0.0,
                "{name} has an inward triangle"
            );
        }
    }
}

/// Every blade triangle of a fan has a twin with the opposite winding.
#[test]
fn fan_is_double_sided() {
    let c = srgb(0.5, 0.5, 0.5);
    for spec in [[5.0, 1.0, 0.5, 0.0, 0.2], [4.0, 1.0, 0.8, 0.3, 0.2]] {
        let tris = build(|m| fan(m, Vec3::ZERO, spec, 0.3, c));
        let facing = |t: &[Vec3; 3]| (t[1] - t[0]).cross(t[2] - t[0]);
        for t in &tris {
            let twin = tris
                .iter()
                .any(|u| u.iter().all(|p| t.contains(p)) && facing(u).dot(facing(t)) < 0.0);
            assert!(twin, "a fan triangle has no reversed twin");
        }
    }
}
