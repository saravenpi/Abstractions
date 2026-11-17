//! Flat-shaded terrain meshes: detail chunks and coarse quadtree nodes, stored in their own frame on the globe.

mod frame;
mod lattice;
mod lod;
mod node;
mod patch;
mod sample;
mod skirt;
mod span;
#[cfg(test)]
mod tests_edges;
#[cfg(test)]
mod tests_globe;
mod tris;

use bevy::prelude::*;

pub use self::frame::Frame;
pub use self::lod::{Lod, next_lod, ring_distance, subsample};
pub use self::node::{fill_node, node_frame, node_mesh};
pub use self::patch::Patch;
pub use self::span::{Span, leaf_coord};
use self::tris::{Style, fill, tone};
use super::shapes::FlatMesh;
use super::{Biome, CHUNK_SIZE, Terrain};

/// Distance between two terrain vertices.
pub const GRID_STEP: f32 = 2.0;

/// Number of vertices along one chunk edge.
pub const GRID_POINTS: usize = (CHUNK_SIZE / GRID_STEP) as usize + 1;

/// Half width of the biome border dither band in world units.
const DITHER_REACH: f32 = 12.0;

/// Samples ground heights at every `stride`-th grid vertex; the first index runs along `x`, the second along `z`.
pub fn chunk_heights(terrain: &Terrain, coord: IVec2, stride: usize) -> Vec<Vec<f32>> {
    Span::leaf(terrain, coord, (GRID_POINTS - 1) / stride).heights()
}

/// Returns the biome of triangle `half` of cell `(i, j)` of `span`, dithered across biome borders.
pub(super) fn dithered(terrain: &Terrain, span: &Span, cell: [usize; 3]) -> Biome {
    let [i, j, half] = cell;
    let jitter = |salt: usize| (tone(i + salt, j, half) - 1.0) / 0.08 * DITHER_REACH;
    let at = span.chart(i, j, 0.5) + Vec2::new(jitter(7919), jitter(104_729));
    terrain.biome(at.x, at.y)
}

/// Returns the vertices of chunk `coord` at `heights`, in the frame of the chunk.
pub fn chunk_patch(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Patch {
    let span = Span::leaf(terrain, coord, heights.len() - 1);
    let frame = Frame::at(terrain, span.center());
    Patch::new(&span, &frame, heights, Style::DETAIL.floor)
}

/// Builds the mesh of chunk `coord` from `heights` sampled at the stride of `lod`, with a per-triangle biome dither.
pub fn chunk_mesh(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>], lod: Lod) -> Mesh {
    let span = Span::leaf(terrain, coord, (GRID_POINTS - 1) / lod.stride());
    let patch = chunk_patch(terrain, coord, heights);
    let mut mesh = FlatMesh::default();
    fill(&mut mesh, &patch, Style::DETAIL, &|i, j, half| {
        dithered(terrain, &span, [i, j, half])
    });
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
