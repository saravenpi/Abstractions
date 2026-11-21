//! Far planets: the coarse sphere drawn for a globe seen from space, and the distance at which
//! the world hands it to the sky.

mod ball;
#[cfg(test)]
mod tests_ball;

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{ChunkId, Face, Globe};
use crate::world::Terrain;
use crate::world::mesh::{Frame, fill_node};
use crate::world::shapes::FlatMesh;

pub use self::ball::{Ball, Map, NODE_CELLS, SIDES, ball, face};

/// Quadtree level of the nodes baked into the far mesh: sixteen per face.
pub const LEVEL: u8 = 2;

/// Cells along one side of a baked node.
pub const CELLS: usize = 16;

/// Distance from the planet centre, in radii, beyond which the quadtree is dropped: the
/// hand-over itself, so the world never draws its coarse far mesh in place of the quadtree on the
/// way out.
pub const ENTER: f64 = LIMIT;

/// Distance from the planet centre, in radii, under which the quadtree comes back. Between it and
/// the hand-over the sky draws the ball of the planet, sunk just under its sea, so the quadtree
/// covers it where both are drawn and it stands in where the quadtree is gone.
pub const LEAVE: f64 = 1.4;

/// Distance from the planet centre, in radii, beyond which the world no longer draws the planet
/// and the sky does. Every far distance of the world and the sky is a number of radii, so the
/// hand-over means the same on a 6 km planet and on a 12,000 km one.
pub const LIMIT: f64 = 1.5;

/// Distance from the planet centre, in radii, up to which the fog and the fade of the world
/// reach at most: far past the hand-over, where nothing of the world is drawn any more.
pub const SIGHT: f64 = 100.0;

/// How a planet is drawn for a focus at some distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Range {
    /// Streamed as a quadtree.
    Near,
    /// Coming back from space: the quadtree is not streamed yet and the sky draws the ball.
    Far,
    /// Too far to be drawn by the world.
    Beyond,
}

/// Returns how to draw a planet whose centre is `radii` planet radii from the focus.
///
/// `far` tells whether the quadtree is dropped now, which it stays down to [`LEAVE`].
pub fn range(radii: f64, far: bool) -> Range {
    if radii > LIMIT {
        Range::Beyond
    } else if radii > [ENTER, LEAVE][usize::from(far)] {
        Range::Far
    } else {
        Range::Near
    }
}

/// Returns the distance in metres from the centre of a planet of `radius` metres beyond which
/// the world no longer draws it and the sky does: [`LIMIT`] radii. The sky draws a body at its
/// true distance and size on the depth of the camera, which has no far plane, so the rings and
/// the moons around the planet need no distance of their own. The world and the sky both read
/// it from here.
pub fn handover(radius: f64) -> f64 {
    LIMIT * radius
}

/// Returns the distance in metres from the centre of a planet of `radius` metres beyond which the
/// sky draws the ball of the planet below the camera: [`LEAVE`] radii, where the quadtree comes
/// back on the way down.
pub fn sky_from(radius: f64) -> f64 {
    LEAVE * radius
}

/// Returns how to draw `globe` for a focus at `at` metres from its centre: as [`range`] says,
/// and not at all past its [`handover`].
pub fn seen(globe: &Globe, at: DVec3, far: bool) -> Range {
    let distance = at.length();
    if distance > globe.handover() {
        Range::Beyond
    } else {
        range(distance / globe.radius(), far)
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
/// flat sea, and no skirts since every node has the same level. The world shows it only while
/// the quadtree has not yet covered the globe.
pub fn bake(globe: &Arc<Globe>) -> Mesh {
    let mut mesh = FlatMesh::default();
    for id in nodes() {
        let terrain = Terrain::on(globe.clone(), globe.face_chart(id.face));
        fill_node(&mut mesh, &terrain, (id, CELLS), &Frame::PLANET, false);
    }
    mesh.into_mesh()
}
