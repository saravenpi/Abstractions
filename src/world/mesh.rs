//! Flat-shaded terrain meshes: detail chunks and coarse quadtree nodes, stored in their own frame on the globe.

mod bands;
mod frame;
mod lattice;
mod lod;
mod node;
mod patch;
mod read;
mod rings;
mod sample;
mod skirt;
mod span;
mod style;
#[cfg(test)]
mod tests_bands;
#[cfg(test)]
mod tests_edges;
#[cfg(test)]
mod tests_globe;
#[cfg(test)]
mod tests_read;
mod tris;
mod tune;

use bevy::prelude::*;

pub use self::frame::Frame;
pub use self::lod::{Lod, ring_distance, subsample};
pub use self::node::{fill_node, node_frame, node_mesh};
pub use self::patch::Patch;
use self::read::scale;
pub use self::rings::Rings;
pub use self::span::{Span, leaf_coord};
use self::style::{Style, tone};
use self::tris::{Cell, fill};
use self::tune::COARSE;
use super::globe::Chart;
use super::lands::{Reading, Verdure};
use super::scatter::Carpet;
use super::shapes::FlatMesh;
use super::{CHUNK_SIZE, Terrain};

/// Distance between two terrain vertices.
pub const GRID_STEP: f32 = 2.0;

/// Number of vertices along one chunk edge.
pub const GRID_POINTS: usize = (CHUNK_SIZE / GRID_STEP) as usize + 1;

/// Period in metres of the turf noise of `turf.wesl`: a whole number of chunks.
pub const TURF_PERIOD: f32 = 256.0;

/// Half width of the biome border dither band in world units.
const DITHER_REACH: f32 = 12.0;

/// Samples ground heights at every `stride`-th grid vertex; the first index runs along `x`, the second along `z`.
pub fn chunk_heights(terrain: &Terrain, coord: IVec2, stride: usize) -> Vec<Vec<f32>> {
    Span::leaf(terrain, coord, (GRID_POINTS - 1) / stride).heights()
}

/// Returns the biome of cell `(i, j)` of `span`, then the biomes of its two triangles, dithered
/// across biome borders, and the channel bed at its middle on a globe.
pub(super) fn dithered(
    terrain: &Terrain,
    span: &Span,
    (i, j): (usize, usize),
    cover: Option<&dyn Fn(Vec2) -> f32>,
) -> Cell {
    let centre = span.chart(i, j, 0.5);
    let near = |half: usize| {
        let jitter = |salt: usize| (tone(i + salt, j, half) - 1.0) / 0.08 * DITHER_REACH;
        centre + Vec2::new(jitter(7919), jitter(104_729))
    };
    let near = [centre, near(0), near(1)];
    let coarse = terrain.chart() != Chart::Flat && scale(span.step()) >= COARSE;
    let glanced = coarse.then(|| terrain.globe().glance(terrain.up(centre.x, centre.y)));
    Cell {
        biomes: match &glanced {
            Some((_, spot)) => terrain.biomes_glanced(near, *spot),
            None => terrain.biomes(near),
        },
        bed: terrain.bed(centre.x, centre.y),
        reading: glanced.map(|(height, spot)| Reading::glanced(height, &spot)),
        cover: cover.map(|cover| cover(centre)),
    }
}

/// Returns the vertices of chunk `coord` at `heights`, in the frame of the chunk, none lower than `floor`.
fn patch_above(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>], floor: f32) -> Patch {
    let span = Span::leaf(terrain, coord, heights.len() - 1);
    let frame = Frame::at(terrain, span.center());
    Patch::new(&span, &frame, heights, floor)
}

/// Returns the vertices of chunk `coord` at `heights`, in the frame of the chunk.
pub fn chunk_patch(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>]) -> Patch {
    patch_above(terrain, coord, heights, f32::NEG_INFINITY)
}

/// Builds the mesh of chunk `coord` from `heights` sampled at the stride of `lod`, with a per-triangle biome dither.
/// The distant level draws a flat sea over the sea bed, like a coarse node.
pub fn chunk_mesh(terrain: &Terrain, coord: IVec2, heights: &[Vec<f32>], lod: Lod) -> Mesh {
    let span = Span::leaf(terrain, coord, (GRID_POINTS - 1) / lod.stride());
    let lands = terrain.globe().lands();
    let style = match lod {
        Lod::Distant => Style::coarse(span.step(), lands),
        _ => Style::detail(lands),
    };
    let style = style.grown(span.step(), Verdure::of(terrain, span.center()));
    let patch = patch_above(terrain, coord, heights, style.floor);
    let mut mesh = FlatMesh::default();
    let carpet = Carpet::of(terrain, coord);
    let seen = (lod != Lod::Distant).then_some(carpet.as_ref()).flatten();
    let chart = &span;
    let turf = seen.map(|carpet| {
        move |i: usize, j: usize| {
            let w = chart.chart(i, j, 0.0);
            let [x, z] = wrap(coord, w);
            [x, z, carpet.at(w)]
        }
    });
    let turf = turf
        .as_ref()
        .map(|t| t as &dyn Fn(usize, usize) -> [f32; 3]);
    let thick = carpet.as_ref().map(|carpet| move |w: Vec2| carpet.at(w));
    let thick = thick.as_ref().map(|t| t as &dyn Fn(Vec2) -> f32);
    fill(&mut mesh, &patch, (style, turf), &|i, j| {
        dithered(terrain, &span, (i, j), thick)
    });
    mesh.into_mesh()
}

/// Returns chart position `w` of chunk `coord` wrapped for the turf noise: the chunk corner taken to its
/// place in a [`TURF_PERIOD`] metre tile, plus the offset of `w` from the corner. Neighbour chunks agree
/// on their shared edge, since the period is a whole number of chunks.
pub fn wrap(coord: IVec2, w: Vec2) -> [f32; 2] {
    let chunks = (TURF_PERIOD / CHUNK_SIZE) as i32;
    let corner = coord.as_vec2() * CHUNK_SIZE;
    let base =
        IVec2::new(coord.x.rem_euclid(chunks), coord.y.rem_euclid(chunks)).as_vec2() * CHUNK_SIZE;
    (base + w - corner).to_array()
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
