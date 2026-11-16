//! Flat-shaded chunk mesh and collider heights sampled from the terrain in world coordinates.

mod lod;
mod skirt;

use bevy::prelude::*;

pub use self::lod::{Lod, next_lod, ring_distance, subsample};
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

/// Samples ground heights at every `stride`-th grid vertex; the first index runs along `x`, the second along `z`.
pub fn chunk_heights(terrain: &Terrain, coord: IVec2, stride: usize) -> Vec<Vec<f32>> {
    let o = coord.as_vec2() * CHUNK_SIZE;
    let step = GRID_STEP * stride as f32;
    (0..=(GRID_POINTS - 1) / stride)
        .map(|i| {
            (0..=(GRID_POINTS - 1) / stride)
                .map(|j| terrain.height(o.x + i as f32 * step, o.y + j as f32 * step))
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
fn push_tri(
    mesh: &mut FlatMesh,
    p: [Vec3; 3],
    key: (usize, usize, usize),
    biome: Biome,
) -> [f32; 4] {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let slope = (1.0 - normal.y * normal.y).max(0.0).sqrt() / normal.y.max(0.01);
    let height = (p[0].y + p[1].y + p[2].y) / 3.0;
    let (r, g, b) = ground_rgb(classify(height, slope, biome), biome);
    let t = tone(key.0, key.1, key.2);
    let color = srgb(r * t, g * t, b * t);
    mesh.tri(p[0], p[1], p[2], color);
    color
}

/// Builds the mesh from a height grid spaced `step` apart, asking `biome_at(i, j, half)` for each triangle's biome.
fn build(heights: &[Vec<f32>], step: f32, biome_at: &dyn Fn(usize, usize, usize) -> Biome) -> Mesh {
    let mut mesh = FlatMesh::default();
    let n = heights.len() - 1;
    let half = CHUNK_SIZE * 0.5;
    let at = |i: usize, j: usize| {
        Vec3::new(
            i as f32 * step - half,
            heights[i][j],
            j as f32 * step - half,
        )
    };
    let mut colors = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            let (p00, p10, p01, p11) = (at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1));
            let lower = push_tri(&mut mesh, [p00, p01, p10], (i, j, 0), biome_at(i, j, 0));
            let upper = push_tri(&mut mesh, [p10, p01, p11], (i, j, 1), biome_at(i, j, 1));
            colors.push([lower, upper]);
        }
    }
    skirt::add(&mut mesh, n, &at, &|i, j, half| colors[i * n + j][half]);
    mesh.into_mesh()
}

/// Builds the mesh of chunk `coord` from `heights` sampled at the stride of `lod`, with a per-triangle biome dither.
pub fn chunk_mesh(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>], lod: Lod) -> Mesh {
    let o = coord.as_vec2() * CHUNK_SIZE;
    let step = GRID_STEP * lod.stride() as f32;
    build(heights, step, &|i, j, half| {
        let jitter = |salt: usize| (tone(i + salt, j, half) - 1.0) / 0.08 * DITHER_REACH;
        let x = o.x + (i as f32 + 0.5) * step + jitter(7919);
        let z = o.y + (j as f32 + 0.5) * step + jitter(104_729);
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
            let (ha, hb) = (chunk_heights(&t, a, 1), chunk_heights(&t, b, 1));
            let last = GRID_POINTS - 1;
            if a.x != b.x {
                assert!((0..=last).all(|j| ha[last][j] == hb[0][j]));
            } else {
                assert!((0..=last).all(|i| ha[i][last] == hb[i][0]));
            }
        }
    }

    /// Each level holds two triangles per cell plus eight skirt triangles per cell of edge length.
    #[test]
    fn mesh_vertices_per_level() {
        let t = Terrain::new(9);
        let full = chunk_heights(&t, IVec2::ZERO, 1);
        for lod in [Lod::Near, Lod::Mid, Lod::Far] {
            let heights = chunk_heights(&t, IVec2::ZERO, lod.stride());
            assert_eq!(heights, subsample(&full, lod.stride()));
            let n = (GRID_POINTS - 1) / lod.stride();
            let mesh = chunk_mesh(&t, IVec2::ZERO, &heights, lod);
            assert_eq!(mesh.count_vertices(), (n * n * 2 + n * 8) * 3);
        }
    }
}
