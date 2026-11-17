//! Detail chunk streaming: which leaves around the focus get props, structures and colliders.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, futures_lite::future::poll_once};

use super::build::generate;
use super::near::Around;
use super::plan::{ring, select_batch};
use super::spawn::{Spawner, spawn_chunk};
use super::{Chunks, DEFAULT_RADIUS, MAX_PENDING, Settled, raytrace};
use crate::settings::{Persistent, Settings};
use crate::world::focus::Focus;
use crate::world::globe::{Chart, ChunkId};
use crate::world::mesh::{Lod, leaf_coord, ring_distance};
use crate::world::{Terrain, WorldReady};

impl Chunks {
    /// Returns the leaves of the deepest level, the only nodes that can carry detail.
    fn leaves(&self, deepest: u8) -> Vec<ChunkId> {
        let mut leaves = self.tree.leaves();
        leaves.retain(|id| id.level == deepest);
        leaves
    }

    /// Despawns chunks and cancels tasks that are no longer leaves or lie beyond `radius` plus one.
    fn drop_far(&mut self, commands: &mut Commands, kept: &HashSet<ChunkId>) {
        let stale: Vec<ChunkId> = self
            .loaded
            .keys()
            .copied()
            .filter(|id| !kept.contains(id))
            .collect();
        for id in stale {
            if let Some(l) = self.loaded.remove(&id) {
                commands.entity(l.entity).try_despawn();
            }
        }
        self.pending.retain(|id, _| kept.contains(id));
    }

    /// Starts generation tasks for the leaves of `wanted`, in order, while there is room.
    fn start(&mut self, wanted: Vec<ChunkId>, around: &Around, spawner: &Spawner) {
        let pool = AsyncComputeTaskPool::get();
        let room = MAX_PENDING.saturating_sub(self.pending.len());
        for id in wanted.into_iter().take(room) {
            let terrain = spawner.view(id.face);
            let lod = Lod::for_ring(ring_distance(around.coord(id), around.center));
            let task = pool.spawn(async move { generate(&terrain, leaf_coord(id), lod) });
            self.pending.insert(id, task);
        }
    }
}

/// Returns what the detail chunks depend on: the focus face and chunk, the radius and the tree.
fn key(chunks: &Chunks, around: &Around, chart: Chart, radius: i32) -> Settled {
    Settled {
        face: match chart {
            Chart::Face { face, .. } => Some(face),
            _ => None,
        },
        center: around.center,
        radius,
        revision: chunks.revision,
    }
}

/// Returns the leaves within `radius` chunks of the focus that have neither a chunk nor a task, nearest first.
fn wanted(chunks: &Chunks, leaves: Vec<ChunkId>, around: &Around, radius: i32) -> Vec<ChunkId> {
    let order: HashMap<IVec2, usize> = ring(around.center, radius).into_iter().zip(0..).collect();
    let mut wanted: Vec<(usize, ChunkId)> = leaves
        .into_iter()
        .filter(|id| !chunks.loaded.contains_key(id) && !chunks.pending.contains_key(id))
        .filter_map(|id| order.get(&around.coord(id)).map(|rank| (*rank, id)))
        .collect();
    wanted.sort_by_key(|(rank, _)| *rank);
    wanted.into_iter().map(|(_, id)| id).collect()
}

/// Starts tasks for missing detail chunks near the focus, drops chunks out of range and moves colliders with the
/// focus.
pub fn stream(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    focus: Focus,
    settings: Option<Res<Persistent<Settings>>>,
    spawner: Spawner,
) {
    let Some(around) = focus
        .planet()
        .and_then(|at| Around::new(&spawner.terrain, at))
    else {
        return;
    };
    let radius = settings.map_or(DEFAULT_RADIUS, |s| s.graphics.render_distance) as i32;
    let key = key(&chunks, &around, spawner.terrain.chart(), radius);
    if chunks.settled == Some(key) {
        return;
    }
    let leaves = chunks.leaves(spawner.terrain.globe().leaf_level());
    let far = |id: &ChunkId| {
        (around.coord(*id) - around.center).length_squared() > (radius + 1) * (radius + 1)
    };
    chunks.drop_far(
        &mut commands,
        &leaves.iter().copied().filter(|id| !far(id)).collect(),
    );
    for (id, loaded) in chunks.loaded.iter_mut() {
        loaded.set_solid(&mut commands, around.solid(*id));
    }
    let wanted = wanted(&chunks, leaves, &around, radius);
    chunks.settled = (wanted.is_empty() && chunks.pending.is_empty()).then_some(key);
    chunks.start(wanted, &around, &spawner);
}

/// Turns the nearest finished chunk tasks into entities, a few per frame once the world is ready.
pub fn collect(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    focus: Focus,
) {
    let Some(around) = focus
        .planet()
        .and_then(|at| Around::new(&spawner.terrain, at))
    else {
        return;
    };
    let done: HashMap<IVec2, ChunkId> = chunks
        .finished()
        .into_iter()
        .map(|id| (around.coord(id), id))
        .collect();
    for coord in select_batch(
        done.keys().copied().collect(),
        around.center,
        spawner.budget(),
    ) {
        let task = chunks.pending.remove(&done[&coord]);
        let Some(mut data) = task.and_then(|mut t| block_on(poll_once(&mut t))) else {
            continue;
        };
        raytrace::prepare(&mut data, spawner.ray_tracing.0);
        let mut loaded = spawn_chunk(&mut commands, &mut spawner, data);
        loaded.set_solid(&mut commands, around.solid(done[&coord]));
        chunks.loaded.insert(done[&coord], loaded);
    }
}

/// Flags the world ready once the chunk under the focus and the chunks that touch it all exist with colliders.
pub fn mark_ready(
    chunks: Res<Chunks>,
    focus: Focus,
    terrain: Res<Terrain>,
    mut ready: ResMut<WorldReady>,
) {
    let Some(around) = focus.planet().and_then(|at| Around::new(&terrain, at)) else {
        return;
    };
    let solid = |id: &ChunkId| chunks.loaded.get(id).is_some_and(|l| l.solid);
    if around.near.iter().all(solid) && !ready.0 {
        ready.0 = true;
    }
}
