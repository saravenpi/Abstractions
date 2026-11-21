//! Coarse quadtree nodes: terrain-only meshes with a flat sea, at any level of a face, or the banded deck
//! of a planet without ground.

use std::f64::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::bands;
use super::frame::Frame;
use super::patch::Patch;
use super::span::Span;
use super::style::Style;
use super::tris::fill;
use super::tune::{SKIRT_FLOOR, SPARSE_LEVELS};
use super::{GRID_POINTS, dithered};
use crate::world::Terrain;
use crate::world::globe::ChunkId;
use crate::world::lands::Verdure;
use crate::world::scatter::Spread;
use crate::world::shapes::FlatMesh;

/// Points along one side of the lattice on which a node reads how thick its turf grows.
const LATTICE: usize = 5;

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
        skirt: [0.0, skirt_depth(terrain, id, n)][usize::from(skirts)],
        ..coarse.grown(span.step(), Verdure::of(terrain, span.center()))
    };
    if !terrain.globe().has_ground() {
        return bands::fill(mesh, &span, frame, style.skirt);
    }
    let patch = Patch::new(&span, frame, &span.heights(), style.floor);
    let spread = Spread::over(
        terrain,
        (span.chart(0, 0, 0.0), span.chart(span.n, span.n, 0.0)),
        LATTICE,
    );
    let thick = spread.as_ref().map(|s| move |w: Vec2| s.at(w));
    let thick = thick.as_ref().map(|t| t as &dyn Fn(Vec2) -> f32);
    fill(mesh, &patch, (style, None), &|i, j| {
        dithered(terrain, &span, (i, j), thick)
    });
}

/// Returns the metres between two vertices of node `id` cut into `n` cells per side: the quarter turn of
/// its face shared evenly, so every node of a level reads the ground alike.
fn spacing(terrain: &Terrain, id: ChunkId, n: usize) -> f32 {
    let across = (1u64 << id.level) as f64 * n as f64;
    (terrain.globe().radius() * FRAC_PI_2 / across) as f32
}

/// Returns how deep in metres the skirts of node `id`, cut into `n` cells per side, hang: a cell
/// of the coarsest neighbour it can have, a node one or two levels up with its own cells, and
/// [`SKIRT_FLOOR`] more. Where a coarse edge cuts a hill that the finer edge follows, the gap is
/// never deeper than the cell of the coarse edge, so the skirt of the higher side always closes it.
pub fn skirt_depth(terrain: &Terrain, id: ChunkId, n: usize) -> f32 {
    let leaf = terrain.globe().leaf_level();
    let up = |rise: u8| {
        let level = id.level.saturating_sub(rise);
        let node = ChunkId { level, ..id };
        spacing(terrain, node, node_cells(leaf.saturating_sub(level)))
    };
    SKIRT_FLOOR + spacing(terrain, id, n).max(up(1)).max(up(2))
}

/// Returns the frame the mesh of quadtree node `id` is stored in: the tangent frame at its middle.
pub fn node_frame(terrain: &Terrain, id: ChunkId) -> Frame {
    Frame::at(terrain, Span::node(terrain, id, 1).center())
}

/// Returns the cells along one side of the mesh of a node `above` levels above the leaves: the
/// leaves get the 8 m grid of a far detail chunk, coarser nodes twice and four times as many cells,
/// and the far nodes of [`SPARSE_LEVELS`] half and a quarter of that again.
pub fn node_cells(above: u8) -> usize {
    let cells = (GRID_POINTS - 1) >> 2u8.saturating_sub(above);
    let sparse = usize::from(above >= SPARSE_LEVELS.0) + usize::from(above >= SPARSE_LEVELS.1);
    cells >> sparse
}

/// Returns the cells along one side of the mesh of quadtree node `id` of the planet of `terrain`.
pub fn cells_of(terrain: &Terrain, id: ChunkId) -> usize {
    match terrain.globe().has_ground() {
        true => node_cells(terrain.globe().leaf_level().saturating_sub(id.level)),
        false => bands::CELLS,
    }
}

/// Builds the terrain-only mesh of quadtree node `id`, with the cells per side of [`cells_of`].
pub fn node_mesh(terrain: &Terrain, id: ChunkId) -> Mesh {
    let mut mesh = FlatMesh::default();
    let frame = node_frame(terrain, id);
    fill_node(
        &mut mesh,
        terrain,
        (id, cells_of(terrain, id)),
        &frame,
        true,
    );
    mesh.into_mesh()
}
