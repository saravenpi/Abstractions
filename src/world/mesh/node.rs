//! Coarse quadtree nodes: terrain-only meshes with a flat sea, at any level of a face, or the banded deck
//! of a planet without ground.

use std::f64::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::bands;
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
/// skirts when `skirts` is set. On a planet without ground it is the smooth banded deck instead. A node
/// whose vertices lie farther apart than a river is wide reads no flow and no pond, and one coarser than
/// the region grids reads the bare land, see [`crate::world::globe::Grain`].
pub fn fill_node(
    mesh: &mut FlatMesh,
    terrain: &Terrain,
    node: (ChunkId, usize),
    frame: &Frame,
    skirts: bool,
) {
    let (id, n) = node;
    let terrain = &terrain.spaced(spacing(terrain, id, n));
    let span = Span::node(terrain, id, n);
    let coarse = Style::coarse(span.step(), terrain.globe().lands());
    let style = Style {
        skirt: [0.0, coarse.skirt][usize::from(skirts)],
        ..coarse
    };
    if !terrain.globe().has_ground() {
        return bands::fill(mesh, &span, frame, style.skirt);
    }
    let patch = Patch::new(&span, frame, &span.heights(), style.floor);
    fill(mesh, &patch, (style, None), &|i, j| {
        dithered(terrain, &span, (i, j))
    });
}

/// Returns the metres between two vertices of node `id` cut into `n` cells per side: the quarter turn of
/// its face shared evenly, so every node of a level reads the ground alike.
fn spacing(terrain: &Terrain, id: ChunkId, n: usize) -> f32 {
    let across = (1u64 << id.level) as f64 * n as f64;
    (terrain.globe().radius() * FRAC_PI_2 / across) as f32
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

/// Builds the terrain-only mesh of quadtree node `id`, with [`node_cells`] cells per side, or the
/// [`bands::CELLS`] of a deck.
pub fn node_mesh(terrain: &Terrain, id: ChunkId) -> Mesh {
    let mut mesh = FlatMesh::default();
    let frame = node_frame(terrain, id);
    let cells = match terrain.globe().has_ground() {
        true => node_cells(terrain.globe().leaf_level().saturating_sub(id.level)),
        false => bands::CELLS,
    };
    fill_node(&mut mesh, terrain, (id, cells), &frame, true);
    mesh.into_mesh()
}
