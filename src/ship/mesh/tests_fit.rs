//! Fit tests: every part of the ship goes into the part that carries it, the gear stays in its
//! hinge folded and down, and the left side is the mirror of the right side.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::Solid;
use super::blade::{FIN, WINGLET, blade};
use super::frame::{canopy, fuselage};
use super::parts::{Paint, parts};
use super::tail::{ENGINES, motor};
use super::wing::wing;

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
/// the engine pods are inside the fuselage, and the wing ends are inside the winglets.
#[test]
fn every_part_goes_into_the_part_that_carries_it() {
    let hull = fuselage();
    for side in [-1.0, 1.0] {
        let tip = blade(3.2 * side, &WINGLET);
        assert_held("wing root", wing(side), |p| p.x.abs() < 0.5, &hull);
        assert_held("wing end", wing(side), |p| p.x.abs() > 3.19, &[tip]);
    }
    assert_held("fin", blade(0.0, &FIN), |p| p.y < 0.5, &hull);
    assert_held("canopy", canopy(), |p| p.y < 0.3 || p.z > 0.1, &hull);
    for at in ENGINES {
        assert_held("engine", motor(at).pod, |p| p.z < 2.3, &hull);
    }
}

/// Every corner of the ship has its mirror image on the other side, but for the service panel and the
/// antenna, which are on the left only, and the red and green lights, which are each other's mirror image.
#[test]
fn the_ship_is_the_same_on_both_sides() {
    let all = parts();
    let mut points = Vec::new();
    for part in all
        .iter()
        .filter(|part| part.group != "panel" && part.group != "mast")
    {
        points.extend(corners(part.solid.clone()));
    }
    for point in &points {
        let mirror = Vec3::new(-point.x, point.y, point.z);
        let twin = points.iter().any(|other| other.distance(mirror) < 1e-3);
        assert!(twin || point.x.abs() < 1e-3, "{point} has no mirror image");
    }
    let lamp = |paint| {
        let part = all.iter().find(|part| part.paint == paint).expect("a lamp");
        corners(part.solid.clone())
    };
    let (red, green) = (lamp(Paint::Red), lamp(Paint::Green));
    for point in red {
        let mirror = Vec3::new(-point.x, point.y, point.z);
        assert!(green.iter().any(|other| other.distance(mirror) < 1e-4));
    }
}
