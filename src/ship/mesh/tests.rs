//! Mesh tests: the ship stays a few thousand triangles, none of them degenerate, the far mesh is
//! cheaper than the near one, and every skin has its material.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::parts::parts;
use super::{Skin, far_mesh, hinges, hull_meshes, lamp_meshes, leg_mesh, material};

/// Returns the positions of a mesh.
fn positions(mesh: &Mesh) -> Vec<Vec3> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(points)) => {
            points.iter().map(|p| Vec3::from(*p)).collect()
        }
        _ => Vec::new(),
    }
}

/// Returns the triangles of a mesh, asserting that it has some and that none is degenerate.
fn triangles(mesh: &Mesh) -> usize {
    let points = positions(mesh);
    assert!(!points.is_empty() && points.len().is_multiple_of(3));
    for tri in points.chunks(3) {
        assert!(
            (tri[1] - tri[0]).cross(tri[2] - tri[0]).length() > 1e-6,
            "{tri:?}"
        );
    }
    points.len() / 3
}

/// The detailed ship is between 1,200 and 3,600 triangles with its three legs, the far mesh is
/// under half of that, and every mesh is made of real triangles.
#[test]
fn ship_is_low_poly_and_has_a_cheap_far_mesh() {
    let (lamps, leg) = (lamp_meshes(), leg_mesh());
    let hull: usize = hull_meshes().iter().map(triangles).sum();
    let rest: usize = lamps.iter().map(triangles).sum::<usize>() + 3 * triangles(&leg);
    let far = triangles(&far_mesh());
    assert!(
        hull + rest > 1200 && hull + rest < 3600,
        "{} triangles",
        hull + rest
    );
    assert!(
        far * 2 < hull + rest,
        "{far} far triangles of {}",
        hull + rest
    );
}

/// The ship is symmetric, sits above its gear feet, and every part of it is wound outward: the
/// volume the triangles of each part enclose is positive, which a part turned inside out, or
/// one with a face the wrong way round, would not give.
#[test]
fn hull_faces_outward() {
    let [hull, _, _, _, _] = hull_meshes();
    let (leg, hinge) = (leg_mesh(), hinges()[0]);
    let points = positions(&hull);
    let (low, high) = points.iter().fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
        (lo.min(*p), hi.max(*p))
    });
    let feet = positions(&leg)
        .iter()
        .map(|p| p.y + hinge.y)
        .fold(f32::MAX, f32::min);
    assert!((feet - super::GEAR[0].y as f32).abs() < 1e-4 && low.y > feet + 0.3);
    assert!(high.x > 3.1 && low.x < -3.1, "{low} {high}");
    for part in parts() {
        let volume: f32 = part
            .solid
            .tris
            .iter()
            .map(|tri| tri[0].dot(tri[1].cross(tri[2])) / 6.0)
            .sum();
        assert!(volume > 0.0, "{} encloses {volume} m3", part.group);
    }
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

/// The hull skin is painted in tones and the far mesh in colours, through vertex colours.
#[test]
fn the_plating_is_painted_in_vertex_colours() {
    let [hull, trim, _, glass, _] = hull_meshes();
    assert!(hull.attribute(Mesh::ATTRIBUTE_COLOR).is_some());
    assert!(glass.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    assert!(trim.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    assert!(far_mesh().attribute(Mesh::ATTRIBUTE_COLOR).is_some());
}
