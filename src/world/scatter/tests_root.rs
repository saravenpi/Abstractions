//! Tests that a chunk is named by whole tiles from the middle of its face, not by 32-bit positions: its
//! random stream and its frame are the same wherever the view is rooted.

use std::sync::Arc;

use bevy::math::DVec2;
use bevy::prelude::*;
use rand::RngExt;

use super::{chunk_center, stream};
use crate::world::globe::{BLOCK, Face, Globe};
use crate::world::mesh::Frame;
use crate::world::{CHUNK_SIZE, Terrain};

/// One chunk draws the same random stream through two roots of its face, another chunk another one,
/// and the tangent frame at its middle is the same frame of the planet.
#[test]
fn a_chunk_is_the_same_through_every_root() {
    let globe = Arc::new(Globe::new(4, 6.371e6, 9.81));
    let dir = Face::PosZ.dir(0.41, -0.3);
    let near = Terrain::near(&globe, dir * globe.radius());
    let moved = near.chart().origin() + DVec2::new(-3.0 * BLOCK, BLOCK);
    let other = Terrain::on(globe.clone(), near.chart().rooted(moved));
    let coord = IVec2::new(7, -19);
    let same = coord + near.shift(CHUNK_SIZE) - other.shift(CHUNK_SIZE);
    let draw = |view: &Terrain, c: IVec2| stream(view, c).random::<u64>();
    assert_eq!(draw(&near, coord), draw(&other, same));
    assert_ne!(draw(&near, coord), draw(&other, coord));
    assert_ne!(draw(&near, coord), draw(&near, coord + IVec2::X));
    let (a, b) = (
        Frame::at(&near, chunk_center(coord)),
        Frame::at(&other, chunk_center(same)),
    );
    assert!(a.origin.distance(b.origin) < 1.0e-6);
    assert!(a.rot.angle_between(b.rot) < 1.0e-9);
}
