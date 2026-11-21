//! The coarse layer: one terrain-only mesh per quadtree node, built off-thread and kept hidden
//! while finer nodes cover it.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::budget::Spend;
use super::build::{NodeData, coarse};
use super::horizon::Bounds;
use super::spawn::Spawner;
use super::stretch::Stretch;
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::Path;

/// Most coarse node tasks running at the same time.
const MAX_BUILDING: usize = 16;

/// Finished coarse nodes turned into entities per frame, the time budget allowing.
const NODE_BUDGET: usize = 8;

/// A spawned coarse mesh.
pub struct Node {
    /// The mesh entity.
    pub entity: Entity,
    /// Whether it is on the draw cut.
    pub visible: bool,
    /// Where it lies on the planet.
    pub bounds: Bounds,
    /// Whether it lies wholly below the horizon of the focus, and is hidden even on the cut.
    pub beyond: bool,
    /// Whether it is too far from the focus to cast or receive a shadow.
    pub shadowless: bool,
}

impl Node {
    /// Applies the visibility of the node to its entity: drawn when it is on the cut and above
    /// the horizon.
    pub fn show(&self, commands: &mut Commands) {
        let visibility = match self.visible && !self.beyond {
            true => Visibility::Inherited,
            false => Visibility::Hidden,
        };
        commands.entity(self.entity).try_insert(visibility);
    }
}

/// Coarse nodes that exist and coarse nodes still being generated.
#[derive(Default)]
pub struct Nodes {
    /// Spawned nodes, shown or hidden.
    pub live: HashMap<ChunkId, Node>,
    /// Nodes being generated.
    pub(super) building: HashMap<ChunkId, Task<NodeData>>,
}

impl Nodes {
    /// Returns true when the mesh of `id` exists.
    pub fn has(&self, id: ChunkId) -> bool {
        self.live.contains_key(&id)
    }

    /// Forgets `id` and cancels its task, and returns its entity for the caller to dispose of.
    pub fn take(&mut self, id: ChunkId) -> Option<Node> {
        self.building.remove(&id);
        self.live.remove(&id)
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
        let room = MAX_BUILDING.saturating_sub(self.building.len());
        for id in wanted.into_iter().take(room) {
            let terrain = spawner.view(id);
            let task = pool.spawn(async move { coarse(&terrain, id) });
            self.building.insert(id, task);
        }
    }

    /// Turns finished tasks into hidden entities, the cheapest of `path` first, within the
    /// stretched count and time budget of the frame. Returns how many it spawned.
    pub(super) fn collect(
        &mut self,
        commands: &mut Commands,
        spawner: &mut Spawner,
        (path, stretch): (&Path, Stretch),
    ) -> usize {
        let done = self.building.iter().filter(|(_, task)| task.is_finished());
        let mut done: Vec<(f64, ChunkId)> = done.map(|(id, _)| (path.cost(*id), *id)).collect();
        done.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut spend = Spend::of(stretch.millis(spawner.shares().nodes));
        let mut spawned = 0;
        for (_, id) in done.into_iter().take(stretch.count(NODE_BUDGET)) {
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
