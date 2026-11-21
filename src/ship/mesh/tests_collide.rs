//! Collider tests: the convex parts the ship collides with are the parts that are drawn. Every
//! point of the drawn surface is inside the colliders or within 20 cm of them, and every corner
//! of a collider is within 20 cm of a drawn corner.

use avian3d::prelude::Collider;
use bevy::prelude::*;

use super::hulls;
use super::parts::parts;
use super::shell::MOUTH;

/// Metres a drawn point may be off the colliders, and a collider corner off the drawing: the
/// chamfers of the heat shield, the sunk sockets and the lights are within it.
const TOLERANCE: f32 = 0.2;

/// Returns the colliders of the airframe.
fn colliders() -> Vec<Collider> {
    let make = |points: Vec<Vec3>| Collider::convex_hull(&points).expect("a solid part");
    hulls().into_iter().map(make).collect()
}

/// Returns the distance from `point` to the nearest collider, zero inside one.
fn distance(colliders: &[Collider], point: Vec3) -> f32 {
    let near =
        |collider: &Collider| collider.distance_to_point(Vec3::ZERO, Quat::IDENTITY, point, true);
    colliders.iter().map(near).fold(f32::MAX, f32::min)
}

/// Returns true for a point of the roof of the fuselage over the mouth of the cockpit.
fn over_the_mouth(corner: &Vec3) -> bool {
    corner.x.abs() < 1e-4 && corner.y > 0.3 && corner.z > MOUTH.0 && corner.z < MOUTH.1
}

/// The corners and the middle of every triangle drawn, but for the aerial and the rung, which
/// are thinner than a hand, are inside the colliders or within [`TOLERANCE`] of them: whatever
/// the eye sees, the hull meets.
#[test]
fn what_is_drawn_collides() {
    let colliders = colliders();
    let thin = |group: &str| group == "mast" || group == "rung";
    for part in parts().iter().filter(|part| !thin(part.group)) {
        for tri in &part.solid.tris {
            let middle = (tri[0] + tri[1] + tri[2]) / 3.0;
            for point in tri.iter().copied().chain([middle]) {
                let off = distance(&colliders, point);
                assert!(off <= TOLERANCE, "{}: {point} is {off} m off", part.group);
            }
        }
    }
}

/// Every corner of a collider is within [`TOLERANCE`] of a corner that is drawn: the colliders are
/// no bigger than the ship. The canopy and the spine, whose feet are inside the hull, and the roof
/// of the fuselage over the open mouth of the cockpit, which is not drawn, are left out.
#[test]
fn colliders_are_no_bigger_than_the_ship() {
    let drawn: Vec<Vec3> = parts()
        .iter()
        .flat_map(|part| {
            part.solid
                .tris
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>()
        })
        .collect();
    for (index, hull) in hulls().iter().enumerate().filter(|(index, _)| *index != 1) {
        for corner in hull.iter().filter(|corner| !over_the_mouth(corner)) {
            let near = drawn
                .iter()
                .map(|p| p.distance(*corner))
                .fold(f32::MAX, f32::min);
            assert!(near <= TOLERANCE, "part {index}: {corner} is {near} m off");
        }
    }
}
