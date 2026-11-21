//! The drawn ground: which terrain mesh draws a place, and the grid its triangles stand on, so what
//! must touch the visible ground reads the triangles the player sees there, not a finer ground.
//!
//! A near detail chunk draws the 2 m grid, a farther one every second or fourth vertex of it, and a
//! coarse node its own grid. Over a hollow a coarser grid is drawn above the finer one, over a bump
//! below it: a body on the 2 m ground beside a coarser mesh sinks into it or floats over it.

use bevy::prelude::*;

use super::Chunks;
use crate::world::globe::ChunkId;
use crate::world::mesh::{GRID_STEP, leaf_coord, leaf_node, node_cells};
use crate::world::{CHUNK_SIZE, Terrain};

/// The grid of the terrain mesh drawn over a place: cells of two flat triangles split along the
/// diagonal from the vertex after the low corner along x to the one after it along z, every vertex
/// at the ground height of its chart position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawn {
    /// Chart position of one vertex of the grid: the low corner of its mesh.
    pub origin: Vec2,
    /// Chart units between two vertices.
    pub step: f32,
}

impl Drawn {
    /// The grid of the finest mesh, the one a near detail chunk draws.
    pub const FINE: Drawn = Drawn {
        origin: Vec2::ZERO,
        step: GRID_STEP,
    };

    /// Returns the grid of a detail chunk `coord` drawn with every `stride`-th vertex of the finest grid.
    pub fn chunk(coord: IVec2, stride: usize) -> Drawn {
        Drawn {
            origin: coord.as_vec2() * CHUNK_SIZE,
            step: GRID_STEP * stride as f32,
        }
    }

    /// Returns the grid of a coarse node `above` levels over the leaves whose low corner is the corner of
    /// chunk `corner`: [`node_cells`] cells across the chunks it covers.
    pub fn node(corner: IVec2, above: u8) -> Drawn {
        let chunks = (1u64 << above) as f32;
        Drawn {
            origin: corner.as_vec2() * CHUNK_SIZE,
            step: CHUNK_SIZE * chunks / node_cells(above) as f32,
        }
    }
}

impl Chunks {
    /// Returns the grid of the terrain mesh drawn over chart position `at` of `terrain`: the detail chunk
    /// shown there at its level, else the finest visible coarse node over it. Nothing where no mesh of
    /// the face of the chart is drawn, or off a face chart.
    pub fn drawn_at(&self, terrain: &Terrain, at: Vec2) -> Option<Drawn> {
        let coord = (at / CHUNK_SIZE).floor().as_ivec2();
        let leaf = leaf_node(terrain, coord)?;
        if let Some(chunk) = self.loaded.get(&leaf).filter(|l| l.shown) {
            return Some(Drawn::chunk(coord, chunk.detail.lod.stride()));
        }
        (0..=leaf.level).find_map(|above| {
            let id = ChunkId {
                level: leaf.level - above,
                x: leaf.x >> above,
                y: leaf.y >> above,
                ..leaf
            };
            let node = self.nodes.live.get(&id).filter(|n| n.visible);
            node.map(|_| Drawn::node(leaf_coord(terrain, id), above))
        })
    }
}
