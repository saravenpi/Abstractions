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
/// `terrain` must be the face chart view of the face of the node. Nodes above the leaf level get
/// a flat sea, and skirts when `skirts` is set.
pub fn fill_node(
    mesh: &mut FlatMesh,
    terrain: &Terrain,
    node: (ChunkId, usize),
    frame: &Frame,
    skirts: bool,
) {
    let (id, n) = node;
    let span = Span::node(terrain, id, n);
    let coarse = Style::coarse(span.step());
    let style = Style {
        floor: [Style::DETAIL.floor, coarse.floor]
            [usize::from(id.level < terrain.globe().leaf_level())],
        skirt: [0.0, coarse.skirt][usize::from(skirts)],
    };
    let patch = Patch::new(&span, frame, &span.heights(), style.floor);
    fill(mesh, &patch, style, &|i, j, half| {
        dithered(terrain, &span, [i, j, half])
    });
}

/// Returns the frame the mesh of quadtree node `id` is stored in: the tangent frame at its middle.
pub fn node_frame(terrain: &Terrain, id: ChunkId) -> Frame {
    Frame::at(terrain, Span::node(terrain, id, 1).center())
}

/// Builds the terrain-only mesh of quadtree node `id`, a patch of `GRID_POINTS` vertices per side.
pub fn node_mesh(terrain: &Terrain, id: ChunkId) -> Mesh {
    let mut mesh = FlatMesh::default();
    let frame = node_frame(terrain, id);
    fill_node(&mut mesh, terrain, (id, GRID_POINTS - 1), &frame, true);
    mesh.into_mesh()
}
