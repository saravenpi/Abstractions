//! Spans: the square of ground one mesh covers, and where its grid vertices are on the planet.

use bevy::prelude::*;

use crate::world::globe::{Chart, ChunkId};
use crate::world::{CHUNK_SIZE, Terrain};

/// Returns the chunk coordinate of the low corner of node `id` in the view `terrain` of its face: the chunk
/// of a leaf level node. Counted in whole chunks from the origin of the view, it names the same ground
/// wherever the view is rooted.
pub fn leaf_coord(terrain: &Terrain, id: ChunkId) -> IVec2 {
    let level = terrain.globe().leaf_level();
    let leaves = 1i32 << level.saturating_sub(id.level);
    let corner = IVec2::new(id.x as i32, id.y as i32) * leaves - IVec2::splat((1i32 << level) / 2);
    corner - terrain.shift(CHUNK_SIZE)
}

/// Returns the leaf level node of chunk `coord` on `face`, or nothing when it lies off the face.
pub fn leaf_node(terrain: &Terrain, coord: IVec2) -> Option<ChunkId> {
    let Chart::Face { face, .. } = terrain.chart() else {
        return None;
    };
    let level = terrain.globe().leaf_level();
    let at = coord + terrain.shift(CHUNK_SIZE) + IVec2::splat((1i32 << level) / 2);
    let inside = at.min_element() >= 0 && at.max_element() < 1i32 << level;
    inside.then_some(ChunkId {
        face,
        level,
        x: at.x as u32,
        y: at.y as u32,
    })
}

/// A square of ground cut into `n` by `n` cells.
///
/// On a cube face the vertices come from the shared lattice of the node, elsewhere from the chart.
#[derive(Clone)]
pub struct Span {
    pub(super) terrain: Terrain,
    origin: Vec2,
    size: f32,
    pub(super) node: Option<ChunkId>,
    pub(super) n: usize,
}

impl Span {
    /// Returns the span of chunk `coord` of the chart of `terrain`, cut into `n` cells per side.
    pub fn leaf(terrain: &Terrain, coord: IVec2, n: usize) -> Self {
        Self {
            terrain: terrain.clone(),
            origin: coord.as_vec2() * CHUNK_SIZE,
            size: CHUNK_SIZE,
            node: leaf_node(terrain, coord),
            n,
        }
    }

    /// Returns the span of quadtree node `id`, cut into `n` cells per side.
    ///
    /// `terrain` must be a face chart view of the face of the node, rooted anywhere.
    pub fn node(terrain: &Terrain, id: ChunkId, n: usize) -> Self {
        let level = terrain.globe().leaf_level();
        let leaves = 1i32 << (level - id.level);
        let corner = leaf_coord(terrain, id);
        Self {
            terrain: terrain.clone(),
            origin: corner.as_vec2() * CHUNK_SIZE,
            size: CHUNK_SIZE * leaves as f32,
            node: Some(id),
            n,
        }
    }

    /// Returns the chart position of vertex `(i, j)`, offset by `shift` cells.
    pub fn chart(&self, i: usize, j: usize, shift: f32) -> Vec2 {
        let step = self.step();
        self.origin + Vec2::new((i as f32 + shift) * step, (j as f32 + shift) * step)
    }

    /// Returns the chart position of the middle of the span.
    pub fn center(&self) -> Vec2 {
        self.origin + Vec2::splat(self.size * 0.5)
    }

    /// Returns the width of one cell in chart units.
    pub fn step(&self) -> f32 {
        self.size / self.n as f32
    }
}
