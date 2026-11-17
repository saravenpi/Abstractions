//! The vertex lattice of the cube sphere: one direction per grid vertex, the same from every face.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

use crate::world::globe::ChunkId;

/// Returns the cube coordinate of lattice line `k`, counted from the cube centre out to `half`.
fn axis(k: f64, half: f64) -> f64 {
    if k.abs() >= half {
        k.signum()
    } else {
        (k / half * FRAC_PI_4).tan()
    }
}

/// Returns the unit direction of vertex `(i, j)` of node `id` cut into `n` cells per side.
///
/// The direction only depends on where the vertex lies on the cube, so two nodes that share a
/// vertex get the same bits, also across a face edge.
pub fn dir(id: ChunkId, i: usize, j: usize, n: usize) -> DVec3 {
    let [normal, u, v] = id.face.axes();
    let half = ((1u64 << id.level) * n as u64) as f64;
    let a = 2.0 * (f64::from(id.x) * n as f64 + i as f64) - half;
    let b = 2.0 * (f64::from(id.y) * n as f64 + j as f64) - half;
    let cube = normal * half + u * a + v * b;
    DVec3::new(axis(cube.x, half), axis(cube.y, half), axis(cube.z, half)).normalize()
}
