//! The coarse layer: one terrain-only mesh per quadtree node, built off-thread and kept hidden
//! while finer nodes cover it.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::build::{NodeData, coarse};
use super::spawn::Spawner;
use super::{Chunks, DEFAULT_RADIUS};
use crate::settings::{Persistent, Settings};
use crate::world::focus::Focus;
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::{Changes, MAX_NODES, View, children, select};

/// Most coarse node tasks running at the same time.
const MAX_BUILDING: usize = 16;

/// Finished coarse nodes turned into entities per frame.
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
    fn missing(&self, id: ChunkId) -> bool {
        !self.has(id) && !self.building.contains_key(&id)
    }

    /// Starts tasks for the nodes of `wanted`, in order, while there is room.
    fn start(&mut self, wanted: Vec<ChunkId>, spawner: &Spawner) {
        let pool = AsyncComputeTaskPool::get();
        let on = spawner.ray_tracing.0;
        let room = MAX_BUILDING.saturating_sub(self.building.len());
        for id in wanted.into_iter().take(room) {
            let terrain = spawner.view(id.face);
            let task = pool.spawn(async move { coarse(&terrain, id, on) });
            self.building.insert(id, task);
        }
    }

    /// Turns finished tasks into hidden entities, a few per frame.
    fn collect(&mut self, commands: &mut Commands, spawner: &mut Spawner) {
        let done = self.building.iter().filter(|(_, task)| task.is_finished());
        let done: Vec<ChunkId> = done.map(|(id, _)| *id).take(NODE_BUDGET).collect();
        for id in done {
            let task = self.building.remove(&id);
            if let Some(data) = task.and_then(|mut task| block_on(poll_once(&mut task))) {
                self.live.insert(id, spawner.coarse(commands, data));
            }
        }
    }
}

impl Chunks {
    /// Applies a selection: merged nodes lose their children, with their detail and their tasks.
    fn reshape(&mut self, commands: &mut Commands, changes: &Changes) {
        for child in changes.merges.iter().flat_map(|id| children(*id)) {
            self.nodes.remove(commands, child);
            self.pending.remove(&child);
            if let Some(loaded) = self.loaded.remove(&child) {
                commands.entity(loaded.entity).try_despawn();
            }
        }
        self.tree.apply(changes);
        self.revision += u64::from(*changes != Changes::default());
    }
}

/// Splits and merges the quadtree around the focus, a few nodes per frame, and builds the mesh
/// of every node. A node only splits once its own mesh exists, so a merge shows it at once.
pub fn grow(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    focus: Focus,
    settings: Option<Res<Persistent<Settings>>>,
    mut spawner: Spawner,
) {
    let Some(at) = focus.planet().filter(|_| !chunks.far.active()) else {
        return;
    };
    let globe = spawner.terrain.globe().clone();
    let rings = settings.map_or(DEFAULT_RADIUS, |s| s.graphics.render_distance);
    let mut view = View {
        focus: at,
        radius: globe.radius(),
        deepest: globe.leaf_level(),
        reach: 0.0,
        budget: MAX_NODES.saturating_sub(chunks.tree.count()) / 4,
    };
    view.reach = f64::from(rings + 1) * view.arc(view.deepest);
    chunks.nodes.collect(&mut commands, &mut spawner);
    let changes = select(&chunks.tree, &view, &|id| chunks.nodes.has(id));
    chunks.reshape(&mut commands, &changes);
    let mut wanted = chunks.tree.leaves();
    wanted.retain(|id| chunks.nodes.missing(*id));
    wanted.sort_by(|a, b| view.distance(*a).total_cmp(&view.distance(*b)));
    chunks.nodes.start(wanted, &spawner);
}
