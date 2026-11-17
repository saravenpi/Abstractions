//! Far planets: the coarse sphere drawn for a globe seen from space.

use std::sync::Arc;

use bevy::prelude::*;

use super::{ChunkId, Face, Globe};
use crate::world::Terrain;
use crate::world::mesh::{Frame, fill_node};
use crate::world::shapes::FlatMesh;

/// Quadtree level of the nodes baked into the far mesh: sixteen per face.
pub const LEVEL: u8 = 2;

/// Cells along one side of a baked node.
pub const CELLS: usize = 16;

/// Distance from the planet centre, in radii, beyond which the far mesh replaces the quadtree.
pub const ENTER: f64 = 3.2;

/// Distance from the planet centre, in radii, under which the quadtree comes back.
pub const LEAVE: f64 = 3.0;

/// Distance from the planet centre, in radii, beyond which the planet is no longer drawn here.
pub const LIMIT: f64 = 20.0;

/// How a planet is drawn for a focus at some distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Range {
    /// Streamed as a quadtree.
    Near,
    /// Drawn as the baked sphere.
    Far,
    /// Too far to be drawn by the world.
    Beyond,
}

/// Returns how to draw a planet whose centre is `radii` planet radii from the focus.
///
/// `far` tells whether it is drawn as the baked sphere now, which it stays down to [`LEAVE`].
pub fn range(radii: f64, far: bool) -> Range {
    if radii > LIMIT {
        Range::Beyond
    } else if radii > [ENTER, LEAVE][usize::from(far)] {
        Range::Far
    } else {
        Range::Near
    }
}

/// Returns the nodes of the far mesh: every node of [`LEVEL`] on the six faces.
pub fn nodes() -> Vec<ChunkId> {
    let side = 1u32 << LEVEL;
    let cells = (0..side).flat_map(|x| (0..side).map(move |y| (x, y)));
    let on = |face: Face| {
        cells.clone().map(move |(x, y)| ChunkId {
            face,
            level: LEVEL,
            x,
            y,
        })
    };
    Face::ALL.into_iter().flat_map(on).collect()
}

/// Builds the mesh of the whole planet in the planet frame: land coloured as on the ground, a
/// flat sea, and no skirts since every node has the same level.
pub fn bake(globe: &Arc<Globe>) -> Mesh {
    let mut mesh = FlatMesh::default();
    for id in nodes() {
        let terrain = Terrain::on(globe.clone(), globe.face_chart(id.face));
        fill_node(&mut mesh, &terrain, (id, CELLS), &Frame::PLANET, false);
    }
    mesh.into_mesh()
}
