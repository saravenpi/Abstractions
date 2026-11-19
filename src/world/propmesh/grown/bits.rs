//! Small closed solids grown plants are dressed with: beads for flowers, fruit and resin, and spikes for
//! thorns and spines.

use bevy::prelude::*;

use crate::world::shapes::FlatMesh;

/// Faces of the octahedron of [`bead`], as indices into its six corners.
const OCTA: [[usize; 3]; 8] = [
    [0, 2, 4],
    [2, 1, 4],
    [1, 3, 4],
    [3, 0, 4],
    [2, 0, 5],
    [1, 2, 5],
    [3, 1, 5],
    [0, 3, 5],
];

/// Faces of the tetrahedron of [`spike`].
const TETRA: [[usize; 3]; 4] = [[0, 1, 2], [0, 1, 3], [1, 2, 3], [2, 0, 3]];

/// Appends the closed solid of `corners` and `faces`, each face wound to look away from the centre.
fn closed(m: &mut FlatMesh, corners: &[Vec3], faces: &[[usize; 3]], colour: [f32; 4]) {
    let centre = corners.iter().copied().sum::<Vec3>() / corners.len() as f32;
    for &[a, b, c] in faces {
        let (a, b, c) = (corners[a], corners[b], corners[c]);
        let out = (b - a).cross(c - a).dot((a + b + c) / 3.0 - centre) >= 0.0;
        match out {
            true => m.tri(a, b, c, colour),
            false => m.tri(a, c, b, colour),
        };
    }
}

/// Adds an octahedron of `radii` centred on `at`: 8 triangles.
pub fn bead(m: &mut FlatMesh, at: Vec3, radii: Vec3, colour: [f32; 4]) {
    let corners = [
        at + Vec3::X * radii.x,
        at - Vec3::X * radii.x,
        at + Vec3::Z * radii.z,
        at - Vec3::Z * radii.z,
        at + Vec3::Y * radii.y,
        at - Vec3::Y * radii.y,
    ];
    closed(m, &corners, &OCTA, colour);
}

/// Adds a three sided spike from `at` along `dir` for `length` metres: 4 triangles.
pub fn spike(m: &mut FlatMesh, at: Vec3, dir: Vec3, length: f32, colour: [f32; 4]) {
    let dir = dir.normalize_or(Vec3::Y);
    let (u, v) = dir.any_orthonormal_pair();
    let r = length * 0.25;
    let ring = |k: f32| {
        let a = k * std::f32::consts::TAU / 3.0;
        at + (u * a.cos() + v * a.sin()) * r
    };
    let corners = [ring(0.0), ring(1.0), ring(2.0), at + dir * length];
    closed(m, &corners, &TETRA, colour);
}
