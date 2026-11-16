//! Chunk streaming: builds terrain chunks off-thread around the anchor and drops far ones.

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::focus::Focus;
use super::mesh::{chunk_heights, chunk_mesh};
use super::props::{PropAssets, build_grass, spawn_props};
use super::scatter::{Scatter, chunk_center, scatter_chunk};
use super::{CHUNK_SIZE, Terrain, WorldReady};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Most chunk tasks running at the same time.
const MAX_PENDING: usize = 12;

/// Chunk radius used when the settings are missing.
const DEFAULT_RADIUS: u32 = 4;

/// Chunks that exist and chunks still being generated.
#[derive(Resource, Default)]
pub struct Chunks {
    loaded: HashMap<IVec2, Entity>,
    pending: HashMap<IVec2, Task<ChunkData>>,
}

impl Chunks {
    /// Forgets every chunk and cancels running tasks.
    pub fn clear(&mut self) {
        self.loaded.clear();
        self.pending.clear();
    }
}

/// Everything a worker thread produces for one chunk.
pub struct ChunkData {
    heights: Vec<Vec<f32>>,
    mesh: Mesh,
    grass: Mesh,
    scatter: Scatter,
}

/// Generates the mesh, collider heights and props of chunk `coord`.
pub fn generate(terrain: &Terrain, coord: IVec2) -> ChunkData {
    let heights = chunk_heights(terrain, coord);
    let scatter = scatter_chunk(terrain, coord);
    ChunkData {
        mesh: chunk_mesh(&heights),
        grass: build_grass(&scatter),
        heights,
        scatter,
    }
}

/// Creates the shared material and prop meshes.
pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(PropAssets::new(&mut meshes, &mut mats));
}

/// Starts tasks for missing chunks near the focus and drops chunks that fell out of range.
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
    let far = |c: &IVec2| (*c - center).length_squared() > (radius + 1) * (radius + 1);
    let stale: Vec<IVec2> = chunks.loaded.keys().copied().filter(far).collect();
    for coord in stale {
        if let Some(e) = chunks.loaded.remove(&coord) {
            commands.entity(e).despawn();
        }
    }
    chunks.pending.retain(|c, _| !far(c));
    let mut wanted: Vec<IVec2> = (-radius..=radius)
        .flat_map(|dx| (-radius..=radius).map(move |dz| center + IVec2::new(dx, dz)))
        .filter(|c| (*c - center).length_squared() <= radius * radius)
        .filter(|c| !chunks.loaded.contains_key(c) && !chunks.pending.contains_key(c))
        .collect();
    wanted.sort_by_key(|c| (*c - center).length_squared());
    let pool = AsyncComputeTaskPool::get();
    for coord in wanted
        .into_iter()
        .take(MAX_PENDING.saturating_sub(chunks.pending.len()))
    {
        let terrain = terrain.clone();
        chunks
            .pending
            .insert(coord, pool.spawn(async move { generate(&terrain, coord) }));
    }
}

/// Turns finished chunk tasks into entities with mesh, collider and props.
pub fn collect(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    assets: Res<PropAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let done: Vec<(IVec2, ChunkData)> = chunks
        .pending
        .iter_mut()
        .filter_map(|(c, task)| block_on(poll_once(task)).map(|d| (*c, d)))
        .collect();
    for (coord, data) in done {
        chunks.pending.remove(&coord);
        let center = chunk_center(coord);
        let scale = Vec3::new(CHUNK_SIZE, 1.0, CHUNK_SIZE);
        let entity = commands
            .spawn((
                Mesh3d(meshes.add(data.mesh)),
                MeshMaterial3d(assets.material.clone()),
                Transform::from_xyz(center.x, 0.0, center.y),
                RigidBody::Static,
                Collider::heightfield(data.heights, scale),
                DespawnOnEnter(Screen::Title),
            ))
            .id();
        spawn_props(
            &mut commands,
            entity,
            &assets,
            &data.scatter,
            meshes.add(data.grass),
        );
        chunks.loaded.insert(coord, entity);
    }
}

/// Flags the world ready once the 3x3 chunks around the focus all exist.
pub fn mark_ready(chunks: Res<Chunks>, focus: Focus, mut ready: ResMut<WorldReady>) {
    let Some(center) = focus.chunk() else {
        return;
    };
    let all = (-1..=1)
        .all(|dx| (-1..=1).all(|dz| chunks.loaded.contains_key(&(center + IVec2::new(dx, dz)))));
    if all && !ready.0 {
        ready.0 = true;
    }
}
