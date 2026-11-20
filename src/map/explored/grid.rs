//! The leaves of the explored ground: a quadtree on the cube-sphere of each body, leaves about
//! 500 m across, named by the keys of [`abstractions_net::leaves`].
//!
//! Directions are unit vectors in the frame that turns with the body. A direction belongs to
//! the face of its largest axis; on the face it is placed by the tangent of its angle, bent
//! back by an arc tangent so leaves stay near one size from the middle of a face to its corner.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

use abstractions_net::leaves::{DEPTH_MAX, key, parts};
use bevy::math::DVec3;

use super::morton::{along, gather, spread};

/// Metres across a leaf at most, which sets the depth of a body.
pub const LEAF_METRES: f64 = 500.0;

/// The normal, the right and the up axis of each of the six faces.
const FACES: [[DVec3; 3]; 6] = [
    [DVec3::X, DVec3::Z, DVec3::Y],
    [DVec3::NEG_X, DVec3::Z, DVec3::Y],
    [DVec3::Y, DVec3::X, DVec3::Z],
    [DVec3::NEG_Y, DVec3::X, DVec3::Z],
    [DVec3::Z, DVec3::X, DVec3::Y],
    [DVec3::NEG_Z, DVec3::X, DVec3::Y],
];

/// Returns the depth of the leaves of a body of `radius` metres: the least depth whose leaf is
/// at most [`LEAF_METRES`] across at the middle of a face.
pub fn depth_for(radius: f64) -> u64 {
    let along = FRAC_PI_2 * radius.max(1.0) / LEAF_METRES;
    (along.log2().ceil().max(1.0) as u64).min(DEPTH_MAX)
}

/// Returns the face of unit direction `dir`.
fn face_of(dir: DVec3) -> usize {
    let abs = dir.abs();
    let axis = match (abs.x >= abs.y, abs.x >= abs.z, abs.y >= abs.z) {
        (true, true, _) => 0,
        (false, _, true) => 1,
        _ => 2,
    };
    2 * axis + usize::from(dir[axis] < 0.0)
}

/// Returns the leaf at `depth` that holds unit direction `dir`.
pub fn leaf_of(dir: DVec3, depth: u64) -> u64 {
    let face = face_of(dir);
    let [normal, right, up] = FACES[face];
    let reach = dir.dot(normal).abs().max(f64::MIN_POSITIVE);
    let n = along(depth);
    let at = |axis: DVec3| {
        let bent = (dir.dot(axis) / reach).atan() / FRAC_PI_4;
        (((bent + 1.0) * 0.5 * n) as u64).min(n as u64 - 1)
    };
    key(
        face as u64,
        depth,
        spread(at(right)) | (spread(at(up)) << 1),
    )
}

/// Returns the unit direction of the centre of the leaf at column `x` and row `y` of `face`.
/// A column or row past the edge of the face continues over the plane of the face.
pub fn direction(face: usize, depth: u64, (x, y): (i64, i64)) -> DVec3 {
    let [normal, right, up] = FACES[face];
    let n = along(depth);
    let tilt = |index: i64| (((index as f64 + 0.5) / n * 2.0 - 1.0) * FRAC_PI_4).tan();
    (normal + right * tilt(x) + up * tilt(y)).normalize()
}

/// Returns the face, the depth and the column and row of `leaf`.
pub fn cell(leaf: u64) -> (usize, u64, (i64, i64)) {
    let (face, depth, morton) = parts(leaf);
    (
        face as usize,
        depth,
        (gather(morton) as i64, gather(morton >> 1) as i64),
    )
}

/// Returns the unit direction of the centre of `leaf`.
pub fn centre(leaf: u64) -> DVec3 {
    let (face, depth, at) = cell(leaf);
    direction(face, depth, at)
}
