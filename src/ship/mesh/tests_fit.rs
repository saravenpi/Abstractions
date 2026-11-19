//! Fit tests: every part of the ship goes into the part that carries it, and the left side is
//! the mirror of the right side.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::frame::{canopy, fin, fuselage, wing};
use super::machinery::{engines, leg};
use super::{GEAR, Solid, hull_meshes};

/// Returns the corners of the triangles of a mesh, three for each triangle.
pub(super) fn positions(mesh: &Mesh) -> Vec<Vec3> {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(points)) => {
            points.iter().map(|p| Vec3::from(*p)).collect()
        }
        _ => Vec::new(),
    }
}

/// Returns the corners of the triangles of a part.
pub(super) fn corners(part: Solid) -> Vec<Vec3> {
    positions(&part.into_mesh())
}

/// Returns true when the ray from `from` along `dir` goes through the triangle `tri`.
pub(super) fn crosses(tri: &[Vec3], from: Vec3, dir: Vec3) -> bool {
    let (ab, ac) = (tri[1] - tri[0], tri[2] - tri[0]);
    let normal = dir.cross(ac);
    let scale = ab.dot(normal);
    if scale.abs() < 1e-9 {
        return false;
    }
    let to = from - tri[0];
    let (u, across) = (to.dot(normal) / scale, to.cross(ab));
    let (v, along) = (dir.dot(across) / scale, ac.dot(across) / scale);
    u >= 0.0 && v >= 0.0 && u + v <= 1.0 && along > 0.0
}

/// Returns true when `point` is inside the closed part whose triangles have `corners`: a ray
/// from it leaves the part through an odd number of triangles.
pub(super) fn inside(corners: &[Vec3], point: Vec3) -> bool {
    let dir = Vec3::new(0.3713, 0.8217, 0.4331);
    let hits = corners.chunks(3).filter(|tri| crosses(tri, point, dir));
    hits.count() % 2 == 1
}

/// Asserts that every corner of `part` that `joins` picks is inside one of the `carriers`,
/// and that there are such corners.
fn assert_held(name: &str, part: Solid, joins: impl Fn(Vec3) -> bool, carriers: &[Solid]) {
    let carriers: Vec<Vec<Vec3>> = carriers.iter().cloned().map(corners).collect();
    let roots: Vec<Vec3> = corners(part).into_iter().filter(|p| joins(*p)).collect();
    assert!(roots.len() >= 3, "{name}: no corner to hold");
    for root in roots {
        let held = carriers.iter().any(|carrier| inside(carrier, root));
        assert!(
            held,
            "{name}: the corner {root} hangs outside what carries it"
        );
    }
}

/// The wing roots, the foot of the fin, the rim and the ends of the canopy and the fronts of
/// the engines are inside the fuselage, and the wing ends are inside the winglets. The roots
/// the wings had before, 0.9 m from the middle, were outside it.
#[test]
fn every_part_goes_into_the_part_that_carries_it() {
    let hull = fuselage();
    let [body, _] = hull.clone().map(corners);
    assert!(inside(&body, Vec3::new(0.0, 0.1, 1.0)) && !inside(&body, Vec3::new(0.0, 2.0, 1.0)));
    for old in [Vec3::new(0.9, -0.1, -0.2), Vec3::new(0.9, -0.1, 2.6)] {
        assert!(!inside(&body, old), "{old} was a wing root in the air");
    }
    for side in [-1.0, 1.0] {
        let [wing, winglet] = wing(side);
        assert_held("wing root", wing.clone(), |p| p.x.abs() < 0.5, &hull);
        assert_held("wing end", wing, |p| p.x.abs() > 3.1, &[winglet]);
    }
    let [engine, _] = engines();
    assert_held("fin", fin(), |p| p.y < 0.6, &hull);
    assert_held("canopy", canopy(), |p| p.y < 0.3 || p.z > 0.1, &hull);
    assert_held("engines", engine, |p| p.z < 2.7, &hull);
}

/// The strut of the nose gear ends inside the fuselage and the two others inside the wings,
/// and every pad stands on the foot the flight uses.
#[test]
fn the_gear_reaches_the_ship() {
    let carriers = [fuselage().to_vec(), wing(-1.0).to_vec(), wing(1.0).to_vec()];
    for (foot, carrier) in GEAR.map(|foot| foot.as_vec3()).into_iter().zip(carriers) {
        let [strut, pad] = leg(foot);
        assert_held("strut", strut, |p| p.y > foot.y + 1.0, &carrier);
        let lowest = corners(pad).iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((lowest - foot.y).abs() < 1e-5, "pad at {lowest}");
    }
}

/// Every corner of every skin has its mirror image on the other side of the ship.
#[test]
fn the_ship_is_the_same_on_both_sides() {
    for mesh in hull_meshes() {
        let points = positions(&mesh);
        for point in &points {
            let mirror = Vec3::new(-point.x, point.y, point.z);
            let twin = points.iter().any(|other| other.distance(mirror) < 1e-4);
            assert!(twin, "{point} has no mirror image");
        }
    }
}
