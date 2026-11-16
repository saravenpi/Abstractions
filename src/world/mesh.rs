//! Flat-shaded chunk mesh and collider heights sampled from the terrain in world coordinates.

use bevy::prelude::*;

use super::biome::ground_rgb;
use super::shapes::{FlatMesh, srgb};
use super::surface::classify;
use super::{Biome, CHUNK_SIZE, Terrain};

/// Distance between two terrain vertices.
pub const GRID_STEP: f32 = 2.0;

/// Number of vertices along one chunk edge.
pub const GRID_POINTS: usize = (CHUNK_SIZE / GRID_STEP) as usize + 1;

/// Half width of the biome border dither band in world units.
const DITHER_REACH: f32 = 12.0;

/// Samples ground heights at every grid vertex; the first index runs along `x`, the second along `z`.
pub fn chunk_heights(terrain: &Terrain, coord: IVec2) -> Vec<Vec<f32>> {
    let o = coord.as_vec2() * CHUNK_SIZE;
    (0..GRID_POINTS)
        .map(|i| {
            (0..GRID_POINTS)
                .map(|j| terrain.height(o.x + i as f32 * GRID_STEP, o.y + j as f32 * GRID_STEP))
                .collect()
        })
        .collect()
}

/// Returns a stable value in `[0.92, 1.08]` for a grid cell and triangle index.
fn tone(i: usize, j: usize, half: usize) -> f32 {
    let h = (i as u32).wrapping_mul(73_856_093)
        ^ (j as u32).wrapping_mul(19_349_663)
        ^ (half as u32 * 83_492_791);
    let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
    0.92 + ((h >> 10) & 0xFF) as f32 / 255.0 * 0.16
}

/// Adds one terrain triangle, coloring it by its centroid height, flat slope and dithered biome.
fn push_tri(mesh: &mut FlatMesh, p: [Vec3; 3], key: (usize, usize, usize), biome: Biome) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let slope = (1.0 - normal.y * normal.y).max(0.0).sqrt() / normal.y.max(0.01);
    let height = (p[0].y + p[1].y + p[2].y) / 3.0;
    let (r, g, b) = ground_rgb(classify(height, slope, biome), biome);
    let t = tone(key.0, key.1, key.2);
    mesh.tri(p[0], p[1], p[2], srgb(r * t, g * t, b * t));
}

/// Builds the mesh from heights, asking `biome_at(i, j, half)` for the biome of each triangle.
fn build(heights: &[Vec<f32>], biome_at: &dyn Fn(usize, usize, usize) -> Biome) -> Mesh {
    let mut mesh = FlatMesh::default();
    let half = CHUNK_SIZE * 0.5;
    let at = |i: usize, j: usize| {
        Vec3::new(
            i as f32 * GRID_STEP - half,
            heights[i][j],
            j as f32 * GRID_STEP - half,
        )
    };
    for i in 0..GRID_POINTS - 1 {
        for j in 0..GRID_POINTS - 1 {
            let (p00, p10, p01, p11) = (at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1));
            push_tri(&mut mesh, [p00, p01, p10], (i, j, 0), biome_at(i, j, 0));
            push_tri(&mut mesh, [p10, p01, p11], (i, j, 1), biome_at(i, j, 1));
        }
    }
    mesh.into_mesh()
}

/// Builds the mesh of chunk `coord`, coloring by biome with a per-triangle dither near biome borders.
pub fn chunk_mesh(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Mesh {
    let o = coord.as_vec2() * CHUNK_SIZE;
    build(heights, &|i, j, half| {
        let jitter = |salt: usize| (tone(i + salt, j, half) - 1.0) / 0.08 * DITHER_REACH;
        let x = o.x + (i as f32 + 0.5) * GRID_STEP + jitter(7919);
        let z = o.y + (j as f32 + 0.5) * GRID_STEP + jitter(104_729);
        terrain.biome(x, z)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Neighbouring chunks agree on the heights of their shared edge.
    #[test]
    fn chunk_edges_match() {
        let t = Terrain::new(9);
        for (a, b) in [
            (IVec2::new(2, 3), IVec2::new(3, 3)),
            (IVec2::new(-1, -1), IVec2::new(-1, 0)),
        ] {
            let (ha, hb) = (chunk_heights(&t, a), chunk_heights(&t, b));
            let last = GRID_POINTS - 1;
            if a.x != b.x {
                assert!((0..=last).all(|j| ha[last][j] == hb[0][j]));
            } else {
                assert!((0..=last).all(|i| ha[i][last] == hb[i][0]));
            }
        }
    }

    /// A chunk mesh holds two triangles per grid cell.
    #[test]
    fn mesh_has_expected_vertices() {
        let t = Terrain::new(9);
        let heights = chunk_heights(&t, IVec2::ZERO);
        let mesh = chunk_mesh(&t, IVec2::ZERO, &heights);
        assert_eq!(
            mesh.count_vertices(),
            (GRID_POINTS - 1) * (GRID_POINTS - 1) * 6
        );
    }
}
