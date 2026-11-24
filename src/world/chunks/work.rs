//! Which loaded chunks rebuild their terrain or their ground cover now.

use std::collections::HashMap;

use bevy::tasks::AsyncComputeTaskPool;

use super::detail::Work;
use super::dig::spawner_mods;
use super::near::Around;
use super::plan::Loaded;
use super::spawn::Spawner;
use crate::world::globe::ChunkId;
use crate::world::mesh::{Rings, leaf_coord};

/// Returns what chunk `l` at `rank` rings should rebuild now, if anything: its terrain when
/// `rings` ask for another level, else its ground cover within a ring of `cover`.
fn stale(l: &Loaded, rank: i32, (rings, cover): (&Rings, i32)) -> Option<Work> {
    if l.detail.busy() {
        return None;
    }
    let next = rings.next(l.detail.lod, rank);
    if next != l.detail.lod {
        return Some(Work::Terrain(next));
    }
    l.detail.turf.wanted(rank, cover).then_some(Work::Cover)
}

/// Starts rebuilds for chunks whose wanted level changed or whose ground cover is missing,
/// nearest to the line of flight first, while `room` allows. A focus that flies fast rebuilds
/// nothing: finer terrain and ground cover would not show, and a chunk that would coarsen is
/// behind and gone in a moment.
pub fn start_work(
    loaded: &mut HashMap<ChunkId, Loaded>,
    around: &Around,
    spawner: &Spawner,
    room: usize,
) {
    if !around.pace.covers() {
        return;
    }
    let want = (&spawner.rings(), spawner.reach().cover());
    let mut todo: Vec<(i32, ChunkId, Work)> = loaded
        .iter()
        .filter_map(|(id, l)| {
            let rank = around.rank(*id);
            stale(l, rank, want).map(|work| (rank, *id, work))
        })
        .collect();
    todo.sort_by_key(|(rank, ..)| *rank);
    let mods = spawner_mods(spawner);
    let pool = AsyncComputeTaskPool::get();
    for (_, id, work) in todo.into_iter().take(room) {
        let terrain = spawner.view(id);
        let coord = leaf_coord(&terrain, id);
        if let Some(l) = loaded.get_mut(&id) {
            l.detail.begin(work, (terrain, coord, mods.clone()), pool);
        }
    }
}
