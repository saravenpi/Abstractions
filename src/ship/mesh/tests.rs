//! Mesh tests: the ship stays small and every part is a real, closed-looking shape.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::{Skin, hinges, hull_meshes, leg_mesh, material, plume_mesh};

/// Returns the positions of a mesh.
fn positions(mesh: &Mesh) -> Vec<Vec3> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(points)) => {
            points.iter().map(|p| Vec3::from(*p)).collect()
        }
        _ => Vec::new(),
    }
}

/// The whole ship stays under 600 triangles, no part is empty and no triangle is degenerate.
#[test]
fn ship_is_low_poly() {
    let meshes = hull_meshes();
    let (plume, leg) = (plume_mesh(), leg_mesh());
    let mut triangles = 0;
    for mesh in meshes.iter().chain([&plume, &leg, &leg, &leg]) {
        let points = positions(mesh);
        assert!(!points.is_empty() && points.len().is_multiple_of(3));
        triangles += points.len() / 3;
        for tri in points.chunks(3) {
            assert!(
                (tri[1] - tri[0]).cross(tri[2] - tri[0]).length() > 1e-5,
                "{tri:?}"
            );
        }
    }
    assert!(triangles > 150 && triangles < 600, "{triangles} triangles");
}

/// The hull is symmetric, sits above its gear feet and its parts face outward.
#[test]
fn hull_faces_outward() {
    let [hull, _, _, _, _] = hull_meshes();
    let (leg, hinge) = (leg_mesh(), hinges()[0]);
    let points = positions(&hull);
    let (low, high) = points.iter().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
        (lo.min(*p), hi.max(*p))
    });
    assert!((low.x + high.x).abs() < 1e-4, "{low} {high}");
    let feet = positions(&leg)
        .iter()
        .map(|p| p.y + hinge.y)
        .fold(f32::MAX, f32::min);
    assert!((feet - super::GEAR[0].y as f32).abs() < 1e-4 && low.y > feet + 0.3);
    let centre = Vec3::new(0.0, 0.1, 0.4);
    let outward = points.chunks(3).filter(|tri| {
        let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]);
        let mid = (tri[0] + tri[1] + tri[2]) / 3.0;
        normal.dot(mid - centre) > 0.0
    });
    assert!(outward.count() * 10 >= points.len() / 3 * 9);
}

/// Only the glow is unlit and only the glass is see-through.
#[test]
fn skins_have_their_materials() {
    for skin in Skin::ALL {
        let made = material(skin);
        assert_eq!(made.unlit, skin == Skin::Glow);
        assert_eq!(made.alpha_mode == AlphaMode::Blend, skin == Skin::Glass);
    }
}
