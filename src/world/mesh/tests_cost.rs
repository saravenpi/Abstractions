//! Tests of what coarse nodes of a home planet cost to build: the world loads about 600 of them before it
//! can start, so each must stay far below the budget.

use std::sync::Arc;

use super::*;
use crate::universe::Universe;
use crate::world::cpu::busy;
use crate::world::globe::{ChunkId, Face};
use crate::world::{Globe, Terrain};

/// The most processor time in seconds a coarse node of the home planet of seed 7 may take.
const BUDGET: f64 = 0.02;

/// A node of level 0 and one of level 5 of the home planet of seed 7 each build in under 20 ms of processor
/// time, once a first node has warmed the shared grids.
#[test]
fn coarse_nodes_build_in_time() {
    let universe = Universe::new(7);
    let globe = Arc::new(Globe::of(&universe.planet(universe.home())));
    globe.drainage();
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let node = |level: u8, x: u32, y: u32| ChunkId {
        face: Face::PosY,
        level,
        x,
        y,
    };
    node_mesh(&terrain, node(3, 1, 1));
    for id in [node(0, 0, 0), node(5, 3, 3)] {
        let start = busy();
        let mesh = node_mesh(&terrain, id);
        let took = busy() - start;
        assert!(mesh.count_vertices() > 0);
        assert!(took < BUDGET, "node {id:?} took {took} s");
    }
}
