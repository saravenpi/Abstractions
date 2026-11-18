//! Coarse quadtree nodes: terrain-only meshes with a flat sea, at any level of a face.

use bevy::prelude::*;

use super::frame::Frame;
use super::patch::Patch;
use super::span::Span;
use super::tris::{Style, fill};
use super::{GRID_POINTS, dithered};
use crate::world::Terrain;
use crate::world::globe::ChunkId;
use crate::world::shapes::FlatMesh;

/// Adds the terrain of quadtree node `id`, cut into `n` cells per side, to `mesh` in `frame`.
///
/// `terrain` must be the face chart view of the face of the node. Every node gets a flat sea, and
/// skirts when `skirts` is set.
pub fn fill_node(
    mesh: &mut FlatMesh,
    terrain: &Terrain,
    node: (ChunkId, usize),
    frame: &Frame,
    skirts: bool,
) {
    let (id, n) = node;
    let span = Span::node(terrain, id, n);
    let coarse = Style::coarse(span.step(), terrain.globe().lands());
    let style = Style {
        skirt: [0.0, coarse.skirt][usize::from(skirts)],
        ..coarse
    };
    let patch = Patch::new(&span, frame, &span.heights(), style.floor);
    fill(mesh, &patch, style, &|i, j| {
        dithered(terrain, &span, (i, j))
    });
}

/// Returns the frame the mesh of quadtree node `id` is stored in: the tangent frame at its middle.
pub fn node_frame(terrain: &Terrain, id: ChunkId) -> Frame {
    Frame::at(terrain, Span::node(terrain, id, 1).center())
}

/// Returns the cells along one side of the mesh of a node `above` levels above the leaves: the
/// leaves get the 8 m grid of a far detail chunk, coarser nodes twice and four times as many cells.
pub fn node_cells(above: u8) -> usize {
    (GRID_POINTS - 1) >> 2u8.saturating_sub(above)
}

/// Builds the terrain-only mesh of quadtree node `id`, with [`node_cells`] cells per side.
pub fn node_mesh(terrain: &Terrain, id: ChunkId) -> Mesh {
    let mut mesh = FlatMesh::default();
    let frame = node_frame(terrain, id);
    let cells = node_cells(terrain.globe().leaf_level().saturating_sub(id.level));
    fill_node(&mut mesh, terrain, (id, cells), &frame, true);
    mesh.into_mesh()
}
