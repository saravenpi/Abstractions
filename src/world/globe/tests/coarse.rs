//! The cost of coarse readers on a planet larger than home, in ground samples: a node too coarse to show a
//! river pays for none, whatever the atlas cells under it hold.

use std::sync::Arc;

use crate::universe::{PlanetRecord, Water};
use crate::world::Terrain;
use crate::world::globe::{ChunkId, Face, Globe, Grain};
use crate::world::mesh::node_mesh;

/// Most a node may cost with rivers, in times the same node without them.
const TIMES: u64 = 3;

/// Most ground samples the root node of a face may take: five for each of its 1,024 cells.
const ROOT: u64 = 5 * 1024;

/// Returns a 40 km world of islands, with rivers or with its water frozen so that nothing flows.
fn planet(rivers: bool) -> Arc<Globe> {
    let water = [Water::Ice, Water::Liquid][usize::from(rivers)];
    let record = PlanetRecord {
        water,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(42, 40_000.0, 22.0).with_record(record))
}

/// Returns the ground samples the mesh of node `id` of a fresh `globe` takes.
fn cost(globe: Arc<Globe>, id: ChunkId) -> u64 {
    let terrain = Terrain::on(globe.clone(), globe.face_chart(id.face));
    let before = Globe::sampled();
    node_mesh(&terrain, id);
    Globe::sampled() - before
}

/// Returns the node of `level` on the +X face of `globe` with the highest ground at its middle, read at a
/// glance so that no cell is left warm for the nodes measured after.
fn highest(globe: &Arc<Globe>, level: u8) -> ChunkId {
    let side = 1u32 << level;
    let nodes = (0..side).flat_map(|x| (0..side).map(move |y| (x, y)));
    let middle = |(x, y): (u32, u32)| {
        let at = |k: u32| (f64::from(k) + 0.5) / f64::from(side) * 2.0 - 1.0;
        globe.height_seen(Face::PosX.dir(at(x), at(y)), Grain::Far)
    };
    let best = nodes.max_by(|a, b| middle(*a).total_cmp(&middle(*b)));
    let (x, y) = best.expect("a face has nodes");
    ChunkId {
        face: Face::PosX,
        level,
        x,
        y,
    }
}

/// On a 40 km world of islands, the root node of a face and a node half way down to the leaves each take at
/// most three times the ground samples they take when nothing flows, and the root at most 5,120.
#[test]
fn coarse_nodes_do_not_pay_for_rivers() {
    let root = ChunkId {
        face: Face::PosX,
        level: 0,
        x: 0,
        y: 0,
    };
    let mid = highest(&planet(false), planet(false).leaf_level() / 2);
    let costs = [root, mid].map(|id| (id.level, cost(planet(false), id), cost(planet(true), id)));
    for (level, dry, wet) in costs {
        println!("level {level}: {wet} samples with rivers, {dry} without");
    }
    for (level, dry, wet) in costs {
        assert!(wet <= TIMES * dry, "level {level}: {wet} against {dry}");
    }
    assert!(
        costs[0].2 <= ROOT,
        "the root node takes {} samples",
        costs[0].2
    );
}
