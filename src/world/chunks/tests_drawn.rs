//! Tests of the drawn grid: it names the very vertices the detail chunks and the coarse nodes are
//! built on, and those vertices stand at the ground height of their chart position.

use std::sync::Arc;

use bevy::prelude::*;

use super::drawn::Drawn;
use crate::world::globe::{ChunkId, Face};
use crate::world::mesh::{GRID_POINTS, Span, chunk_heights, leaf_coord, leaf_node, node_cells};
use crate::world::{Globe, Terrain};

/// Returns the view of a 6 km globe through the chart of its +Y face.
fn view() -> Terrain {
    let globe = Arc::new(Globe::new(9, 6000.0, 22.0));
    let chart = globe.face_chart(Face::PosY);
    Terrain::on(globe, chart)
}

/// Returns the node `above` levels over leaf `leaf`.
fn up(leaf: ChunkId, above: u8) -> ChunkId {
    ChunkId {
        level: leaf.level - above,
        x: leaf.x >> above,
        y: leaf.y >> above,
        ..leaf
    }
}

/// Asserts that vertex `(i, j)` of `span` lies at the chart position `grid` gives it, within a millimetre.
fn same(span: &Span, grid: Drawn, (i, j): (usize, usize)) {
    let built = span.chart(i, j, 0.0);
    let named = grid.origin + Vec2::new(i as f32, j as f32) * grid.step;
    assert!(
        built.distance(named) < 1e-3,
        "vertex {i},{j}: {built} built, {named} named"
    );
}

/// Every level of a detail chunk is drawn on the grid [`Drawn::chunk`] names, its vertices at the ground
/// height of their chart position, so a body reading the ground on that grid stands on the triangles drawn.
#[test]
fn detail_chunks_are_drawn_on_their_grid() {
    let terrain = view();
    for coord in [IVec2::new(3, -5), IVec2::new(-7, 2)] {
        for stride in [1, 2, 4] {
            let n = (GRID_POINTS - 1) / stride;
            let span = Span::leaf(&terrain, coord, n);
            let grid = Drawn::chunk(coord, stride);
            let heights = chunk_heights(&terrain, coord, stride);
            for (i, j) in (0..=n).flat_map(|i| (0..=n).map(move |j| (i, j))) {
                same(&span, grid, (i, j));
                let at = grid.origin + Vec2::new(i as f32, j as f32) * grid.step;
                let gap = (heights[i][j] - terrain.height(at.x, at.y)).abs();
                assert!(
                    gap < 1e-3,
                    "stride {stride}, vertex {i},{j}: {gap} m off the ground"
                );
            }
        }
    }
}

/// A coarse node over a chunk, at any level up to three over the leaves, is drawn on the grid
/// [`Drawn::node`] names: 8 m up to two levels over the leaves, 16 m at three.
#[test]
fn coarse_nodes_are_drawn_on_their_grid() {
    let terrain = view();
    let leaf = leaf_node(&terrain, IVec2::new(3, -5)).expect("a leaf on the face");
    let steps = [8.0, 8.0, 8.0, 16.0];
    for above in 0..=leaf.level.min(3) {
        let id = up(leaf, above);
        let n = node_cells(above);
        let span = Span::node(&terrain, id, n);
        let grid = Drawn::node(leaf_coord(&terrain, id), above);
        assert_eq!(
            grid.step,
            steps[usize::from(above)],
            "{above} levels over the leaves"
        );
        for (i, j) in (0..=n)
            .step_by(3)
            .flat_map(|i| (0..=n).step_by(5).map(move |j| (i, j)))
        {
            same(&span, grid, (i, j));
        }
    }
}
