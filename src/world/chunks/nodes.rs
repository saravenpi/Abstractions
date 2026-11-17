//! The coarse layer: one terrain-only mesh per quadtree node, built off-thread and kept hidden
//! while finer nodes cover it.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::budget::Spend;
use super::build::{NodeData, coarse};
use super::spawn::Spawner;
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::View;

/// Most coarse node tasks running at the same time.
const MAX_BUILDING: usize = 16;

/// Finished coarse nodes turned into entities per frame, the time budget allowing.
const NODE_BUDGET: usize = 8;

/// Marks the entity of a coarse node or of the far planet, whose ray tracing twin follows its
/// visibility.
#[derive(Component, Debug, Clone, Copy)]
pub struct Coarse;

/// A spawned coarse mesh.
pub struct Node {
    /// The mesh entity.
    pub entity: Entity,
    /// Its mesh, kept to add and remove the ray tracing twin.
    pub mesh: Handle<Mesh>,
    /// Whether it is drawn now.
    pub visible: bool,
}

/// Coarse nodes that exist and coarse nodes still being generated.
#[derive(Default)]
pub struct Nodes {
    /// Spawned nodes, shown or hidden.
    pub live: HashMap<ChunkId, Node>,
    building: HashMap<ChunkId, Task<NodeData>>,
}

impl Nodes {
    /// Returns true when the mesh of `id` exists.
    pub fn has(&self, id: ChunkId) -> bool {
        self.live.contains_key(&id)
    }

    /// Despawns `id` or cancels its task.
    pub fn remove(&mut self, commands: &mut Commands, id: ChunkId) {
        self.building.remove(&id);
        if let Some(node) = self.live.remove(&id) {
            commands.entity(node.entity).try_despawn();
        }
    }

    /// Despawns every node and cancels every task.
    pub fn clear(&mut self, commands: &mut Commands) {
        for node in self.live.values() {
            commands.entity(node.entity).try_despawn();
        }
        *self = Self::default();
    }

    /// Returns true when `id` has neither a mesh nor a task.
    pub(super) fn missing(&self, id: ChunkId) -> bool {
        !self.has(id) && !self.building.contains_key(&id)
    }

    /// Returns true when no node is being built.
    pub(super) fn idle(&self) -> bool {
        self.building.is_empty()
    }

    /// Starts tasks for the nodes of `wanted`, in order, while there is room.
    pub(super) fn start(&mut self, wanted: Vec<ChunkId>, spawner: &Spawner) {
        let pool = AsyncComputeTaskPool::get();
        let on = spawner.ray_tracing.0;
        let room = MAX_BUILDING.saturating_sub(self.building.len());
        for id in wanted.into_iter().take(room) {
            let terrain = spawner.view(id.face);
            let task = pool.spawn(async move { coarse(&terrain, id, on) });
            self.building.insert(id, task);
        }
    }

    /// Turns finished tasks into hidden entities, nearest to the focus of `view` first, within
    /// the count and the time budget of the frame. Returns how many it spawned.
    pub(super) fn collect(
        &mut self,
        commands: &mut Commands,
        spawner: &mut Spawner,
        view: &View,
    ) -> usize {
        let done = self.building.iter().filter(|(_, task)| task.is_finished());
        let mut done: Vec<(f64, ChunkId)> = done.map(|(id, _)| (view.distance(*id), *id)).collect();
        done.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut spend = Spend::of(spawner.shares().nodes);
        let mut spawned = 0;
        for (_, id) in done.into_iter().take(NODE_BUDGET) {
            if !spend.more() {
                break;
            }
            let task = self.building.remove(&id);
            if let Some(data) = task.and_then(|mut task| block_on(poll_once(&mut task))) {
                self.live.insert(id, spawner.coarse(commands, data));
                spawned += 1;
            }
        }
        spawned
    }
}
