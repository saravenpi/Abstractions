//! Low-poly solids appended to a [`FlatMesh`]: cones, prisms, frustums, fans and jittered icospheres.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::shapes::FlatMesh;

#[cfg(test)]
pub(crate) mod tests;

/// Golden ratio used by the icosahedron vertices.
const PHI: f32 = std::f32::consts::GOLDEN_RATIO;

/// Icosahedron faces as indices into [`icosa_vertices`].
pub(in crate::world) const FACES: [[usize; 3]; 20] = [
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
pub(in crate::world) fn icosa_vertices() -> [Vec3; 12] {
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

/// Number of sides of every frustum.
const SIDES: u32 = 6;

/// Shape of a fan of blades: count, horizontal reach, tip rise, mid-blade arch and blade width.
pub type Fan = [f32; 5];

/// Darkens a linear color by a small amount that depends on the face index.
fn varied(color: [f32; 4], i: usize) -> [f32; 4] {
    let k = 0.93 + 0.07 * ((i * 5) % 4) as f32 / 3.0;
    [color[0] * k, color[1] * k, color[2] * k, color[3]]
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
        let tone = varied(color, i as usize);
        mesh.tri(ring(i + 1), ring(i), tip, tone);
        mesh.tri(ring(i), ring(i + 1), base, tone);
    }
}

/// Adds a closed upright prism with its bottom circle centered on `base`.
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
        mesh.tri(a, b, base, color);
        mesh.tri(d, c, base + Vec3::Y * height, color);
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
    for (n, [a, b, c]) in FACES.into_iter().enumerate() {
        mesh.tri(verts[a], verts[b], verts[c], varied(color, n));
    }
}

/// Adds a closed six-sided tube from `from` to `to`, with the `radii` at the start and the end.
pub fn frustum(mesh: &mut FlatMesh, from: Vec3, to: Vec3, radii: Vec2, color: [f32; 4]) {
    let axis = (to - from).normalize_or_zero();
    let u = axis.any_orthonormal_vector();
    let v = u.cross(axis);
    let ring = |i: u32, at: Vec3, r: f32| {
        let a = i as f32 / SIDES as f32 * TAU;
        at + (u * a.cos() + v * a.sin()) * r
    };
    for i in 0..SIDES {
        let (a, b) = (ring(i, from, radii.x), ring(i + 1, from, radii.x));
        let (c, d) = (ring(i, to, radii.y), ring(i + 1, to, radii.y));
        let tone = varied(color, i as usize);
        mesh.tri(b, a, c, tone);
        mesh.tri(b, c, d, tone);
        mesh.tri(a, b, from, tone);
        mesh.tri(d, c, to, tone);
    }
}

/// Adds blades radiating from `base`, each visible from both sides and bent at mid-length when `arch` is set.
pub fn fan(mesh: &mut FlatMesh, base: Vec3, spec: Fan, yaw: f32, color: [f32; 4]) {
    let [count, reach, rise, arch, width] = spec;
    let mut both = |a: Vec3, b: Vec3, c: Vec3| {
        mesh.tri(a, b, c, color);
        mesh.tri(b, a, c, color);
    };
    for k in 0..count as u32 {
        let a = yaw + k as f32 / count * TAU;
        let dir = Vec3::new(a.cos(), 0.0, a.sin());
        let side = Vec3::new(-a.sin(), 0.0, a.cos()) * width * 0.5;
        let tip = base + dir * reach + Vec3::Y * rise;
        if arch == 0.0 {
            both(base - side, base + side, tip);
            continue;
        }
        let mid = base + dir * reach * 0.5 + Vec3::Y * (rise * 0.5 + arch);
        both(base - side, base + side, mid + side * 0.7);
        both(base - side, mid + side * 0.7, mid - side * 0.7);
        both(mid - side * 0.7, mid + side * 0.7, tip);
    }
}
