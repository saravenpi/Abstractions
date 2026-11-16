//! Chunk streaming: builds terrain chunks off-thread around the anchor and drops far ones.

mod build;
mod detail;
mod plan;
mod raytrace;
mod spawn;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use self::build::{ChunkData, generate};
use self::detail::{Probe, refine};
use self::plan::{Loaded, needs_collider, ring, select_batch};
pub use self::raytrace::{add_instances, rebuild, retarget};
use self::spawn::{Spawner, spawn_chunk};
use super::focus::Focus;
use super::mesh::{Lod, ring_distance};
use super::props::PropAssets;
use super::{Terrain, WorldReady};
use crate::settings::{Persistent, Settings};

/// Most chunk tasks running at the same time.
const MAX_PENDING: usize = 12;

/// Finished chunks turned into entities per frame once the world is ready.
const FRAME_BUDGET: usize = 2;

/// Chunk radius used when the settings are missing.
const DEFAULT_RADIUS: u32 = 4;

/// Chunks that exist and chunks still being generated.
#[derive(Resource, Default)]
pub struct Chunks {
    loaded: HashMap<IVec2, Loaded>,
    pending: HashMap<IVec2, Task<ChunkData>>,
    settled: Option<(IVec2, i32)>,
}

impl Chunks {
    /// Forgets every chunk and cancels running tasks.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Despawns every chunk and cancels running tasks, so the streamer builds them all again.
    pub fn rebuild(&mut self, commands: &mut Commands) {
        for loaded in self.loaded.values() {
            commands.entity(loaded.entity).try_despawn();
        }
        self.clear();
    }

    /// Returns the chunks whose generation task is done.
    fn finished(&self) -> Vec<IVec2> {
        let done = self.pending.iter().filter(|(_, task)| task.is_finished());
        done.map(|(c, _)| *c).collect()
    }

    /// Despawns chunks and cancels tasks that lie beyond `radius` plus one from `center`.
    fn drop_far(&mut self, commands: &mut Commands, center: IVec2, radius: i32) {
        let far = |c: &IVec2| (*c - center).length_squared() > (radius + 1) * (radius + 1);
        let stale: Vec<IVec2> = self.loaded.keys().copied().filter(far).collect();
        for coord in stale {
            if let Some(l) = self.loaded.remove(&coord) {
                commands.entity(l.entity).despawn();
            }
        }
        self.pending.retain(|c, _| !far(c));
    }
}

/// Creates the shared material.
pub fn setup_assets(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(PropAssets::new(&mut mats, &mut images));
}

/// Starts tasks for missing chunks near the focus, drops chunks out of range and moves colliders with the focus.
pub fn stream(
    mut commands: Commands,
    terrain: Res<Terrain>,
    mut chunks: ResMut<Chunks>,
    focus: Focus,
    settings: Option<Res<Persistent<Settings>>>,
) {
    let Some(center) = focus.chunk() else {
        return;
    };
    let radius = settings.map_or(DEFAULT_RADIUS, |s| s.graphics.render_distance) as i32;
    if chunks.settled == Some((center, radius)) {
        return;
    }
    chunks.drop_far(&mut commands, center, radius);
    for (coord, loaded) in chunks.loaded.iter_mut() {
        loaded.set_solid(&mut commands, needs_collider(*coord, center));
    }
    let wanted: Vec<IVec2> = ring(center, radius)
        .into_iter()
        .filter(|c| !chunks.loaded.contains_key(c) && !chunks.pending.contains_key(c))
        .collect();
    if wanted.is_empty() && chunks.pending.is_empty() {
        chunks.settled = Some((center, radius));
    }
    let pool = AsyncComputeTaskPool::get();
    let room = MAX_PENDING.saturating_sub(chunks.pending.len());
    for coord in wanted.into_iter().take(room) {
        let terrain = terrain.clone();
        let lod = Lod::for_ring(ring_distance(coord, center));
        chunks.pending.insert(
            coord,
            pool.spawn(async move { generate(&terrain, coord, lod) }),
        );
    }
}

/// Turns the nearest finished chunk tasks into entities, a few per frame once the world is ready.
pub fn collect(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    focus: Focus,
    probe: Probe,
) {
    let Some(center) = focus.chunk() else {
        return;
    };
    for coord in select_batch(chunks.finished(), center, spawner.budget()) {
        let Some(mut data) = chunks
            .pending
            .remove(&coord)
            .and_then(|mut t| block_on(poll_once(&mut t)))
        else {
            continue;
        };
        raytrace::prepare(&mut data, spawner.ray_tracing.0);
        let mut loaded = spawn_chunk(&mut commands, &mut spawner, coord, data);
        loaded.set_solid(&mut commands, needs_collider(coord, center));
        chunks.loaded.insert(coord, loaded);
    }
    refine(
        &mut commands,
        &mut chunks.loaded,
        center,
        &mut spawner,
        &probe,
    );
}

/// Flags the world ready once the 3x3 chunks around the focus all exist with colliders.
pub fn mark_ready(chunks: Res<Chunks>, focus: Focus, mut ready: ResMut<WorldReady>) {
    let Some(center) = focus.chunk() else {
        return;
    };
    let all = (-1..=1).all(|dx| {
        (-1..=1).all(|dz| {
            chunks
                .loaded
                .get(&(center + IVec2::new(dx, dz)))
                .is_some_and(|l| l.solid)
        })
    });
    if all && !ready.0 {
        ready.0 = true;
    }
}
