//! Ground patches draped on the terrain mesh: grids of quads with every vertex on the visible ground, cut
//! finer where a sharp crease of the mesh runs between their corners.

use bevy::prelude::*;

use crate::world::structures::brush::Build;

/// Returns the chunk-space point on the visible ground at `p`, raised by `lift`.
fn ground(b: &Build, p: Vec2, lift: f32) -> Vec3 {
    b.world_pt(p, b.mesh_y(p) + lift)
}

/// How far below its corners the terrain mesh may rise under the middle of a ground triangle before its quads
/// are cut finer. The corners float 0.12 m over the mesh, so the middle still stays above it.
const DIP: f32 = 0.08;

/// Most times the quads of a ground patch are halved each way to follow a sharp crease.
const CUTS: u32 = 1;

/// Returns the point at `(u, v)` of the quad between four world corners.
fn corner(c: [Vec2; 4], u: f32, v: f32) -> Vec2 {
    c[0].lerp(c[1], u).lerp(c[3].lerp(c[2], u), v)
}

/// Returns how deep the middle of the lowest triangle of `c` cut into `n.0` by `n.1` quads lies under the
/// terrain mesh, the way [`patch`] splits each quad.
fn dip(b: &Build, c: [Vec2; 4], n: (u32, u32)) -> f32 {
    let at = |i: u32, j: u32| corner(c, i as f32 / n.0 as f32, j as f32 / n.1 as f32);
    let cells = (0..n.0).flat_map(|i| (0..n.1).map(move |j| (i, j)));
    let tri = |t: [Vec2; 3]| {
        let mean = t.iter().map(|p| b.mesh_y(*p)).sum::<f32>() / 3.0;
        b.mesh_y((t[0] + t[1] + t[2]) / 3.0) - mean
    };
    cells
        .map(|(i, j)| {
            let q = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
            tri([q[0], q[1], q[2]]).max(tri([q[0], q[2], q[3]]))
        })
        .fold(0.0, f32::max)
}

/// Returns by how much the quads of `c` cut into `n` must be halved so that no triangle dips under a
/// crease of the terrain mesh by more than [`DIP`].
pub fn fineness(b: &Build, c: [Vec2; 4], n: (u32, u32)) -> u32 {
    let tries = (0..CUTS).map(|k| 1 << k);
    tries
        .into_iter()
        .find(|f| dip(b, c, (n.0 * f, n.1 * f)) <= DIP)
        .unwrap_or(1 << CUTS)
}

/// Draws the ground between four world corners as `n.0` by `n.1` quads with every vertex on the terrain mesh.
pub fn patch(b: &mut Build, c: [Vec2; 4], n: (u32, u32), coat: ([f32; 3], f32)) {
    let color = b.color(coat.0, 1.0);
    let at = |i: u32, j: u32| {
        let p = corner(c, i as f32 / n.0 as f32, j as f32 / n.1 as f32);
        ground(b, p, coat.1)
    };
    let grid: Vec<Vec<Vec3>> = (0..=n.0)
        .map(|i| (0..=n.1).map(|j| at(i, j)).collect())
        .collect();
    for (lo, hi) in grid.iter().zip(&grid[1..]) {
        for j in 0..n.1 as usize {
            b.quad_out([lo[j], hi[j], hi[j + 1], lo[j + 1]], Vec3::Y, color);
        }
    }
}
