//! Flat-shaded chunk mesh and collider heights sampled from the terrain in world coordinates.

use bevy::prelude::*;

use super::shapes::{FlatMesh, srgb};
use super::surface::classify;
use super::{CHUNK_SIZE, Surface, Terrain};

/// Distance between two terrain vertices.
pub const GRID_STEP: f32 = 2.0;

/// Number of vertices along one chunk edge.
pub const GRID_POINTS: usize = (CHUNK_SIZE / GRID_STEP) as usize + 1;

/// Returns the world position of the corner of chunk `coord` with the lowest `x` and `z`.
fn origin(coord: IVec2) -> Vec2 {
    coord.as_vec2() * CHUNK_SIZE
}

/// Samples ground heights at every grid vertex; the first index runs along `x`, the second along `z`.
pub fn chunk_heights(terrain: &Terrain, coord: IVec2) -> Vec<Vec<f32>> {
    let o = origin(coord);
    (0..GRID_POINTS)
        .map(|i| {
            (0..GRID_POINTS)
                .map(|j| terrain.height(o.x + i as f32 * GRID_STEP, o.y + j as f32 * GRID_STEP))
                .collect()
        })
        .collect()
}

/// Returns the vertex color of a surface, lightened or darkened by a per-triangle factor near 1.
fn surface_color(surface: Surface, tone: f32) -> [f32; 4] {
    let (r, g, b) = match surface {
        Surface::Water => (0.16, 0.22, 0.26),
        Surface::Sand => (0.82, 0.74, 0.52),
        Surface::Grass => (0.30, 0.56, 0.24),
        Surface::Rock => (0.46, 0.45, 0.46),
    };
    srgb(r * tone, g * tone, b * tone)
}

/// Returns a stable value in `[0.92, 1.08]` for a grid cell and triangle index.
fn tone(i: usize, j: usize, half: usize) -> f32 {
    let h = (i as u32).wrapping_mul(73_856_093)
        ^ (j as u32).wrapping_mul(19_349_663)
        ^ (half as u32 * 83_492_791);
    let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
    0.92 + ((h >> 10) & 0xFF) as f32 / 255.0 * 0.16
}

/// Adds one terrain triangle, coloring it by its centroid height and flat slope.
fn push_tri(mesh: &mut FlatMesh, p: [Vec3; 3], i: usize, j: usize, half: usize) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let slope = (1.0 - normal.y * normal.y).max(0.0).sqrt() / normal.y.max(0.01);
    let height = (p[0].y + p[1].y + p[2].y) / 3.0;
    let color = surface_color(classify(height, slope), tone(i, j, half));
    mesh.tri(p[0], p[1], p[2], color);
}

/// Builds the mesh of chunk `coord`, with vertices relative to the chunk center on the XZ plane.
pub fn chunk_mesh(heights: &[Vec<f32>]) -> Mesh {
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
            push_tri(&mut mesh, [p00, p01, p10], i, j, 0);
            push_tri(&mut mesh, [p10, p01, p11], i, j, 1);
        }
    }
    mesh.into_mesh()
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
        let mesh = chunk_mesh(&chunk_heights(&t, IVec2::ZERO));
        assert_eq!(
            mesh.count_vertices(),
            (GRID_POINTS - 1) * (GRID_POINTS - 1) * 6
        );
    }
}
