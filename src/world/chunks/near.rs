//! Chunk coordinates around the focus: every leaf seen through the chart of the focus face.

use std::f64::consts::FRAC_PI_2;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::plan::needs_collider;
use crate::world::globe::quadtree::leaf_at;
use crate::world::globe::{Chart, ChunkId};
use crate::world::mesh::{leaf_coord, ring_distance};
use crate::world::{CHUNK_SIZE, Globe, Terrain};

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
    /// Rings added to every chunk for the height of the focus above the ground, one per chunk
    /// width: seen from the air everything is farther.
    pub lift: i32,
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
        let globe = terrain.globe();
        let ground = f64::from(globe.height(focus.normalize_or(DVec3::Y)).max(0.0));
        let above = focus.length() - globe.radius() - ground;
        Some(Self {
            lift: (above / leaf_arc(globe)).floor().clamp(0.0, 1.0e6) as i32,
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

    /// Returns how far leaf `id` is from the focus in rings of chunks, the height of the focus
    /// above the ground included.
    pub fn rank(&self, id: ChunkId) -> i32 {
        ring_distance(self.coord(id), self.center) + self.lift
    }

    /// Returns true when leaf `id` must have colliders: it is the leaf under the focus, touches
    /// it, or lies within the collider rings in the chart of the focus face.
    pub fn solid(&self, id: ChunkId) -> bool {
        self.near.contains(&id) || needs_collider(self.coord(id), self.center)
    }
}

/// Returns the width in metres of a detail chunk in the middle of a cube face of `globe`.
pub fn leaf_arc(globe: &Globe) -> f64 {
    globe.radius() * FRAC_PI_2 / f64::from(1u32 << globe.leaf_level())
}
