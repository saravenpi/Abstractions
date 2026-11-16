//! Vertical skirts hung along chunk edges so neighbours of different detail show no cracks.

use bevy::prelude::*;

use crate::world::shapes::FlatMesh;

/// How far a skirt hangs below the terrain edge.
const DROP: f32 = 3.0;

/// Adds one wall below the edge from `a` to `b`, facing `out`, in `color`.
fn wall(mesh: &mut FlatMesh, a: Vec3, b: Vec3, color: [f32; 4], out: Vec3) {
    let (a2, b2) = (a - Vec3::Y * DROP, b - Vec3::Y * DROP);
    if (b - a).cross(b2 - a).dot(out) > 0.0 {
        mesh.tri(a, b, b2, color);
        mesh.tri(a, b2, a2, color);
    } else {
        mesh.tri(a, b2, b, color);
        mesh.tri(a, a2, b2, color);
    }
}

/// Adds the four skirts of a grid of `n` by `n` cells.
///
/// `at(i, j)` gives a grid point; `color(i, j, half)` gives the color of a triangle.
pub fn add(
    mesh: &mut FlatMesh,
    n: usize,
    at: &dyn Fn(usize, usize) -> Vec3,
    color: &dyn Fn(usize, usize, usize) -> [f32; 4],
) {
    for k in 0..n {
        wall(mesh, at(0, k), at(0, k + 1), color(0, k, 0), -Vec3::X);
        wall(mesh, at(n, k), at(n, k + 1), color(n - 1, k, 1), Vec3::X);
        wall(mesh, at(k, 0), at(k + 1, 0), color(k, 0, 0), -Vec3::Z);
        wall(mesh, at(k, n), at(k + 1, n), color(k, n - 1, 1), Vec3::Z);
    }
}
