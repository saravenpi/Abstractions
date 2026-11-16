//! Low-poly solids appended to a [`FlatMesh`]: cones, prisms and jittered icospheres.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::shapes::FlatMesh;

/// Golden ratio used by the icosahedron vertices.
const PHI: f32 = 1.618_034;

/// Icosahedron faces as indices into [`icosa_vertices`].
const FACES: [[usize; 3]; 20] = [
    [0, 11, 5],
    [0, 5, 1],
    [0, 1, 7],
    [0, 7, 10],
    [0, 10, 11],
    [1, 5, 9],
    [5, 11, 4],
    [11, 10, 2],
    [10, 7, 6],
    [7, 1, 8],
    [3, 9, 4],
    [3, 4, 2],
    [3, 2, 6],
    [3, 6, 8],
    [3, 8, 9],
    [4, 9, 5],
    [2, 4, 11],
    [6, 2, 10],
    [8, 6, 7],
    [9, 8, 1],
];

/// Returns the twelve unit vertices of an icosahedron.
fn icosa_vertices() -> [Vec3; 12] {
    [
        Vec3::new(-1.0, PHI, 0.0),
        Vec3::new(1.0, PHI, 0.0),
        Vec3::new(-1.0, -PHI, 0.0),
        Vec3::new(1.0, -PHI, 0.0),
        Vec3::new(0.0, -1.0, PHI),
        Vec3::new(0.0, 1.0, PHI),
        Vec3::new(0.0, -1.0, -PHI),
        Vec3::new(0.0, 1.0, -PHI),
        Vec3::new(PHI, 0.0, -1.0),
        Vec3::new(PHI, 0.0, 1.0),
        Vec3::new(-PHI, 0.0, -1.0),
        Vec3::new(-PHI, 0.0, 1.0),
    ]
    .map(Vec3::normalize)
}

/// Adds a cone with its base circle centered on `base`, pointing up.
pub fn cone(mesh: &mut FlatMesh, base: Vec3, size: Vec2, sides: u32, color: [f32; 4]) {
    let (radius, height) = (size.x, size.y);
    let ring = |i: u32| {
        let a = i as f32 / sides as f32 * TAU;
        base + Vec3::new(a.cos(), 0.0, a.sin()) * radius
    };
    let tip = base + Vec3::Y * height;
    for i in 0..sides {
        mesh.tri(ring(i + 1), ring(i), tip, color);
        mesh.tri(ring(i), ring(i + 1), base, color);
    }
}

/// Adds an upright prism with its bottom circle centered on `base`.
pub fn prism(mesh: &mut FlatMesh, base: Vec3, size: Vec2, sides: u32, color: [f32; 4]) {
    let (radius, height) = (size.x, size.y);
    let at = |i: u32, y: f32| {
        let a = i as f32 / sides as f32 * TAU;
        base + Vec3::new(a.cos() * radius, y, a.sin() * radius)
    };
    for i in 0..sides {
        let (a, b) = (at(i, 0.0), at(i + 1, 0.0));
        let (c, d) = (at(i, height), at(i + 1, height));
        mesh.tri(b, a, c, color);
        mesh.tri(b, c, d, color);
    }
}

/// Returns a stable pseudo-random value in `[-1, 1]` for a vertex index and seed.
fn jitter(index: usize, seed: u32) -> f32 {
    let h = (index as u32 ^ seed.wrapping_mul(0x9E37_79B9)).wrapping_mul(0x85EB_CA6B);
    let h = (h ^ (h >> 13)).wrapping_mul(0xC2B2_AE35);
    ((h >> 8) & 0xFFFF) as f32 / 32767.5 - 1.0
}

/// Adds a jittered icosahedron scaled by `radii` and centered on `center`.
pub fn icosa(mesh: &mut FlatMesh, center: Vec3, radii: Vec3, shape: (f32, u32), color: [f32; 4]) {
    let (wobble, seed) = shape;
    let verts: Vec<Vec3> = icosa_vertices()
        .iter()
        .enumerate()
        .map(|(i, v)| center + *v * radii * (1.0 + jitter(i, seed) * wobble))
        .collect();
    for [a, b, c] in FACES {
        mesh.tri(verts[a], verts[b], verts[c], color);
    }
}
