//! Skirts hung along chunk edges so neighbours of different detail show no cracks.

use bevy::prelude::*;

use crate::world::shapes::FlatMesh;

/// Adds one wall from the edge `top` down to `bottom`, facing `out`, in `color`.
fn wall(mesh: &mut FlatMesh, top: [Vec3; 2], bottom: [Vec3; 2], color: [f32; 4], out: Vec3) {
    let ([a, b], [a2, b2]) = (top, bottom);
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
/// `at(i, j)` gives a grid point, `color(i, j, half)` the color of a triangle and `drop(i, j)`
/// the offset from a grid point to the bottom of its skirt.
pub fn add(
    mesh: &mut FlatMesh,
    n: usize,
    at: &dyn Fn(usize, usize) -> Vec3,
    color: &dyn Fn(usize, usize, usize) -> [f32; 4],
    drop: &dyn Fn(usize, usize) -> Vec3,
) {
    let edge = |a: [usize; 2], b: [usize; 2]| {
        let top = [at(a[0], a[1]), at(b[0], b[1])];
        (top, [top[0] + drop(a[0], a[1]), top[1] + drop(b[0], b[1])])
    };
    for k in 0..n {
        let (top, bottom) = edge([0, k], [0, k + 1]);
        wall(mesh, top, bottom, color(0, k, 0), -Vec3::X);
        let (top, bottom) = edge([n, k], [n, k + 1]);
        wall(mesh, top, bottom, color(n - 1, k, 1), Vec3::X);
        let (top, bottom) = edge([k, 0], [k + 1, 0]);
        wall(mesh, top, bottom, color(k, 0, 0), -Vec3::Z);
        let (top, bottom) = edge([k, n], [k + 1, n]);
        wall(mesh, top, bottom, color(k, n - 1, 1), Vec3::Z);
    }
}
