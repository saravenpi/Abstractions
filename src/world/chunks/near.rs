//! Chunk coordinates around the focus: every leaf seen through the chart of the focus face.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::plan::needs_collider;
use crate::world::globe::quadtree::leaf_at;
use crate::world::globe::{Chart, ChunkId};
use crate::world::mesh::leaf_coord;
use crate::world::{CHUNK_SIZE, Terrain};

/// The chart of the focus face and the chunk the focus is in.
///
/// Leaves of other faces get the coordinate of their middle in this chart, continued past its
/// edge, so rings around the focus also run across a face edge. Near a cube corner two leaves
/// may share a coordinate and a coordinate may hold no leaf.
pub struct Around {
    terrain: Terrain,
    /// Chunk that holds the focus.
    pub center: IVec2,
    /// The leaf under the focus and the leaves that touch it, across face edges too.
    pub near: Vec<ChunkId>,
}

impl Around {
    /// Returns the coordinates around a focus at planet position `focus`, seen through the face
    /// chart of `terrain`. Other charts have no leaves and give nothing.
    pub fn new(terrain: &Terrain, focus: DVec3) -> Option<Self> {
        if !matches!(terrain.chart(), Chart::Face { .. }) {
            return None;
        }
        let at = terrain.locate(focus);
        let under = leaf_at(focus.normalize_or(DVec3::Y), terrain.globe().leaf_level());
        let steps = (-1..=1).flat_map(|dx| (-1..=1).map(move |dy| (dx, dy)));
        Some(Self {
            terrain: terrain.clone(),
            center: (Vec2::new(at.x, at.z) / CHUNK_SIZE).floor().as_ivec2(),
            near: steps
                .filter_map(|(dx, dy)| under.neighbor(dx, dy))
                .collect(),
        })
    }

    /// Returns the chunk coordinate of leaf `id` in the chart of the focus face.
    pub fn coord(&self, id: ChunkId) -> IVec2 {
        match self.terrain.chart() {
            Chart::Face { face, .. } if face == id.face => leaf_coord(id),
            chart => (chart.locate(id.dir(0.5, 0.5)) / CHUNK_SIZE)
                .floor()
                .as_ivec2(),
        }
    }

    /// Returns true when leaf `id` must have colliders: it is the leaf under the focus, touches
    /// it, or lies within the collider rings in the chart of the focus face.
    pub fn solid(&self, id: ChunkId) -> bool {
        self.near.contains(&id) || needs_collider(self.coord(id), self.center)
    }
}
