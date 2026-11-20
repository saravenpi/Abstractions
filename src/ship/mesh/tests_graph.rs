//! The graph of the parts of the ship, for the tests of attachment: which corners they share,
//! and which parts hold which.

use std::collections::HashMap;

use bevy::prelude::*;

use super::frame::{canopy, fuselage};
use super::parts::Part;
use super::tests_fit::{corners, inside};

/// A corner of a triangle, rounded to a tenth of a millimetre.
pub(super) type Key = [i32; 3];

/// Returns the key of a corner.
pub(super) fn key(p: Vec3) -> Key {
    [p.x, p.y, p.z].map(|v| (v * 1.0e4).round() as i32)
}

/// Returns the corners of the triangles of a part, three for each triangle.
pub(super) fn triangles(part: &Part) -> Vec<Vec3> {
    part.solid.tris.iter().flatten().copied().collect()
}

/// Returns the closed volumes that stand for a part when something is held inside it: the hull
/// and the glass are open over the cockpit, so the closed outlines stand for them.
pub(super) fn volumes(part: &Part) -> Vec<Vec<Vec3>> {
    match part.group {
        "body" | "nose" => fuselage().into_iter().map(corners).collect(),
        "canopy" => vec![corners(canopy())],
        _ => vec![triangles(part)],
    }
}

/// Returns the root of the set of `i` in the sets of `parent`.
pub(super) fn find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        let root = find(parent, parent[i]);
        parent[i] = root;
    }
    parent[i]
}

/// Returns the box of a set of points.
fn bounds(points: &[Vec3]) -> (Vec3, Vec3) {
    let start = (Vec3::MAX, Vec3::MIN);
    points
        .iter()
        .fold(start, |(lo, hi), p| (lo.min(*p), hi.max(*p)))
}

/// Returns true when a corner of `a` is inside a volume of `b`.
pub(super) fn holds(volumes: &[Vec<Vec3>], points: &HashMap<Key, Vec3>) -> bool {
    volumes.iter().any(|volume| {
        let (lo, hi) = bounds(volume);
        let within = |p: &&Vec3| p.cmpge(lo).all() && p.cmple(hi).all();
        points.values().filter(within).any(|p| inside(volume, *p))
    })
}
