//! Detail chunk streaming: which leaves around the focus get props and colliders.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, futures_lite::future::poll_once};

use super::budget::Spend;
use super::dig::{generate_mods, spawner_mods};
use super::near::Around;
use super::plan::batch;
use super::scene::Scene;
use super::spawn::{Spawner, spawn_chunk};
use super::{Chunks, Settled};
use crate::world::globe::{Chart, ChunkId};
use crate::world::mesh::leaf_coord;

impl Chunks {
    /// Returns the leaves of the deepest level, the only nodes that can carry detail.
    fn leaves(&self, deepest: u8) -> Vec<ChunkId> {
        let mut leaves = self.tree.leaves();
        leaves.retain(|id| id.level == deepest);
        leaves
    }

    /// Despawns chunks and cancels tasks that are not `kept`: no longer leaves, or out of range.
    fn drop_far(&mut self, commands: &mut Commands, kept: &HashSet<ChunkId>) {
        let stale: Vec<ChunkId> = self
            .loaded
            .keys()
            .copied()
            .filter(|id| !kept.contains(id))
            .collect();
        self.stamp += stale.len() as u64;
        for id in stale {
            if let Some(l) = self.loaded.remove(&id) {
                commands.entity(l.entity).try_despawn();
            }
        }
        self.pending.retain(|id, _| kept.contains(id));
    }

    /// Starts generation tasks for the leaves of `wanted`, in order, while there is room; the
    /// leaves that must be solid under the focus start whatever the room. A chunk is built at
    /// the level and with the ground cover its place beside the line of flight will ask for,
    /// as far as the pace of the focus allows.
    fn start(&mut self, wanted: Vec<ChunkId>, around: &Around, spawner: &Spawner) {
        let pool = AsyncComputeTaskPool::get();
        let (reach, rings) = (spawner.reach(), spawner.rings());
        let mods = spawner_mods(spawner);
        let room = reach.tasks().saturating_sub(self.pending.len());
        let fits = |(i, id): &(usize, ChunkId)| *i < room || around.solid(*id);
        for (_, id) in wanted.into_iter().enumerate().filter(fits) {
            let terrain = spawner.view(id);
            let coord = leaf_coord(&terrain, id);
            let rank = around.rank(id);
            let lod = rings.lod(rank).max(around.pace.finest());
            let turfed = around.pace.covers() && rank <= reach.cover() + 1;
            let mods = mods.clone();
            let build = async move { generate_mods(&terrain, coord, (lod, turfed), &mods) };
            self.pending.insert(id, pool.spawn(build));
        }
    }
}

/// Returns what the detail chunks depend on: the focus face, the lane and the tree.
fn key(chunks: &Chunks, around: &Around, chart: Chart, rings: i32) -> Settled {
    Settled {
        face: match chart {
            Chart::Face { face, .. } => Some(face),
            _ => None,
        },
        lane: around.lane,
        rings,
        revision: chunks.revision,
    }
}

/// Returns the leaves of the lane that have neither a chunk nor a task, cheapest first.
fn wanted(chunks: &Chunks, leaves: Vec<ChunkId>, around: &Around) -> Vec<ChunkId> {
    let order: HashMap<IVec2, usize> = around.lane.cells().into_iter().zip(0..).collect();
    let mut wanted: Vec<(usize, ChunkId)> = leaves
        .into_iter()
        .filter(|id| !chunks.loaded.contains_key(id) && !chunks.pending.contains_key(id))
        .filter_map(|id| order.get(&around.coord(id)).map(|rank| (*rank, id)))
        .collect();
    wanted.sort_by_key(|(rank, _)| *rank);
    wanted.into_iter().map(|(_, id)| id).collect()
}

/// Starts tasks for missing detail chunks along the lane of the focus, drops chunks out of range
/// and moves colliders with the focus.
pub fn stream(mut commands: Commands, mut chunks: ResMut<Chunks>, scene: Scene, spawner: Spawner) {
    let Some(around) = scene.at().and_then(|at| chunks.around(&spawner, at)) else {
        return;
    };
    let key = key(
        &chunks,
        &around,
        spawner.terrain.chart(),
        spawner.reach().rings,
    );
    if chunks.settled == Some(key) {
        return;
    }
    let leaves = chunks.leaves(spawner.terrain.globe().leaf_level());
    let kept = leaves.iter().copied().filter(|id| around.keeps(*id));
    chunks.drop_far(&mut commands, &kept.collect());
    for (id, loaded) in chunks.loaded.iter_mut() {
        loaded.set_solid(&mut commands, around.solid(*id));
    }
    let wanted = wanted(&chunks, leaves, &around);
    chunks.settled = (wanted.is_empty() && chunks.pending.is_empty()).then_some(key);
    chunks.start(wanted, &around, &spawner);
}

/// Turns finished chunk tasks into entities, the cheapest of the lane first, within the count
/// and the time budget of the frame. The chunks that must be solid under the focus always go
/// through.
pub fn collect(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    scene: Scene,
) {
    let Some(around) = scene.at().and_then(|at| chunks.around(&spawner, at)) else {
        return;
    };
    let coords = chunks
        .finished()
        .into_iter()
        .map(|id| (around.coord(id), id));
    let done: HashMap<IVec2, ChunkId> = coords.collect();
    let reach = spawner.reach();
    let stretch = chunks.stretch;
    let count = [reach.tasks(), stretch.count(reach.per_frame())][usize::from(spawner.ready.0)];
    let mut spend = Spend::of(stretch.millis(spawner.shares().chunks));
    let costs = done.keys().map(|c| (around.lane.cost(*c), *c)).collect();
    for coord in batch(costs, around.center, count) {
        let id = done[&coord];
        if !around.solid(id) && !spend.more() {
            continue;
        }
        let task = chunks.pending.remove(&id);
        let Some(data) = task.and_then(|mut t| block_on(poll_once(&mut t))) else {
            continue;
        };
        let mut loaded = spawn_chunk(&mut commands, &mut spawner, data);
        loaded.set_solid(&mut commands, around.solid(id));
        chunks.loaded.insert(id, loaded);
        chunks.touch();
    }
}
