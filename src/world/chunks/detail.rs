//! Per-chunk level of detail: terrain mesh swaps and ground cover on demand.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Chunks;
use super::budget::Spend;
use super::build::{ground_cover, terrain_mesh};
use super::near::Around;
use super::plan::Loaded;
use super::raytrace;
use super::spawn::Spawner;
use super::turf::Turf;
use crate::world::Terrain;
use crate::world::focus::Focus;
use crate::world::globe::ChunkId;
use crate::world::mesh::{Lod, Rings, leaf_coord};

/// Most terrain meshes and ground covers rebuilt at the same time.
const MAX_REMESH: usize = 4;

/// Detail state of one loaded chunk.
pub struct Detail {
    lod: Lod,
    /// The grass and flowers of the chunk.
    pub turf: Turf,
    structure: Option<(Entity, Collider)>,
    remesh: Option<(Lod, Task<Mesh>)>,
}

impl Detail {
    /// Starts at level `lod` with its ground cover and the structure child and its collider.
    pub fn new(lod: Lod, turf: Turf, structure: Option<(Entity, Collider)>) -> Self {
        Self {
            lod,
            turf,
            structure,
            remesh: None,
        }
    }

    /// Returns the structure child and its collider, when the chunk has solid structures.
    pub fn structure(&self) -> Option<&(Entity, Collider)> {
        self.structure.as_ref()
    }

    /// Starts the task that does `work` for the chunk at `coord` of the chart `terrain`, with the
    /// ray tracing flag `on`.
    fn begin(
        &mut self,
        work: Work,
        (terrain, coord): (Terrain, IVec2),
        on: bool,
        pool: &AsyncComputeTaskPool,
    ) {
        match work {
            Work::Terrain(lod) => {
                let task = pool.spawn(async move { terrain_mesh(&terrain, coord, lod, on) });
                self.remesh = Some((lod, task));
            }
            Work::Cover => self
                .turf
                .start(pool.spawn(async move { ground_cover(&terrain, coord, on) })),
        }
    }

    /// Returns true while a task rebuilds the terrain or the ground cover of the chunk.
    fn busy(&self) -> bool {
        self.remesh.is_some() || self.turf.building()
    }
}

/// Swaps in a finished terrain mesh, the budget allowing.
fn swap_finished(
    commands: &mut Commands,
    l: &mut Loaded,
    spawner: &mut Spawner,
    spend: &mut Spend,
) {
    let Some((lod, task)) = l.detail.remesh.as_mut() else {
        return;
    };
    if !task.is_finished() || !spend.more() {
        return;
    }
    let lod = *lod;
    let Some(mesh) = block_on(poll_once(task)) else {
        return;
    };
    l.detail.remesh = None;
    l.detail.lod = lod;
    let handle = spawner.meshes.add(mesh);
    raytrace::swap(commands, l.entity, handle, spawner.ray_tracing.0);
}

/// What a loaded chunk should rebuild: its terrain at another level, or its ground cover.
enum Work {
    /// The terrain mesh at this level.
    Terrain(Lod),
    /// The ground cover.
    Cover,
}

/// Returns what chunk `l` at `rank` rings should rebuild now, if anything.
fn stale(l: &Loaded, rank: i32, rings: &Rings, cover: i32) -> Option<Work> {
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
/// nearest first, while `room` allows.
fn start_work(
    loaded: &mut HashMap<ChunkId, Loaded>,
    around: &Around,
    spawner: &Spawner,
    room: usize,
) {
    let (reach, rings) = (spawner.reach(), spawner.rings());
    let mut todo: Vec<(i32, ChunkId, Work)> = loaded
        .iter()
        .filter_map(|(id, l)| {
            let rank = around.rank(*id);
            stale(l, rank, &rings, reach.cover()).map(|work| (rank, *id, work))
        })
        .collect();
    todo.sort_by_key(|(rank, ..)| *rank);
    let pool = AsyncComputeTaskPool::get();
    for (_, id, work) in todo.into_iter().take(room) {
        let (terrain, on) = (spawner.view(id.face), spawner.ray_tracing.0);
        if let Some(l) = loaded.get_mut(&id) {
            l.detail.begin(work, (terrain, leaf_coord(id)), on, pool);
        }
    }
}

/// Runs the per-frame detail work for every loaded chunk: ground cover follows the rings and
/// terrain meshes follow their level, within the swap budget of the frame.
pub fn refine(
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
    let cover = spawner.reach().cover();
    let mut spend = Spend::of(spawner.shares().swaps);
    let mut busy = 0;
    for (id, l) in chunks.loaded.iter_mut() {
        let rings = (around.rank(*id), cover);
        let entity = l.entity;
        let turf = &mut l.detail.turf;
        turf.tend(&mut commands, entity, rings, (&mut spawner, &mut spend));
        swap_finished(&mut commands, l, &mut spawner, &mut spend);
        busy += usize::from(l.detail.busy());
    }
    let room = MAX_REMESH.saturating_sub(busy);
    start_work(&mut chunks.loaded, &around, &spawner, room);
}
