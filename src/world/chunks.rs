//! Chunk streaming: builds terrain chunks off-thread around the anchor and drops far ones.

mod plan;

use std::collections::HashMap;

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use self::plan::{Loaded, needs_collider, ring, select_batch};
use super::focus::Focus;
use super::mesh::{chunk_heights, chunk_mesh};
use super::propmesh::merge_props;
use super::props::{Cover, PropAssets, body_collider, build_grass, spawn_props};
use super::scatter::{Placement, chunk_center, scatter_chunk};
use super::structures::{StructureData, build_chunk, origins_near, spawn_structures};
use super::{CHUNK_SIZE, Terrain, WorldReady};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Most chunk tasks running at the same time.
const MAX_PENDING: usize = 12;

/// Finished chunks turned into entities per frame once the world is ready.
const FRAME_BUDGET: usize = 2;

/// Radius around a structure origin kept free of plants.
const CLEARING: f32 = 11.0;

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

/// Everything a worker thread produces for one chunk.
pub struct ChunkData {
    heights: Collider,
    mesh: Mesh,
    grass: Option<Mesh>,
    props: Option<Mesh>,
    bodies: Option<Collider>,
    structures: Option<StructureData>,
}

/// Generates the mesh, collider and merged props of chunk `coord`.
pub fn generate(terrain: &Terrain, coord: IVec2) -> ChunkData {
    let heights = chunk_heights(terrain, coord);
    let mut scatter = scatter_chunk(terrain, coord);
    let taken = origins_near(terrain, coord, CLEARING);
    let center = chunk_center(coord);
    let free = |p: &Placement| {
        let at = center + Vec2::new(p.pos.x, p.pos.z);
        taken.iter().all(|o| o.distance(at) > CLEARING)
    };
    scatter.props.retain(free);
    scatter.grass.retain(free);
    let mesh = chunk_mesh(terrain, coord, &heights);
    let scale = Vec3::new(CHUNK_SIZE, 1.0, CHUNK_SIZE);
    ChunkData {
        mesh,
        grass: (!scatter.grass.is_empty()).then(|| build_grass(&scatter)),
        props: (!scatter.props.is_empty()).then(|| merge_props(&scatter.props)),
        bodies: body_collider(&scatter.props),
        structures: build_chunk(terrain, coord),
        heights: Collider::heightfield(heights, scale),
    }
}

/// Creates the shared material.
pub fn setup_assets(mut commands: Commands, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.insert_resource(PropAssets::new(&mut mats));
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
        chunks
            .pending
            .insert(coord, pool.spawn(async move { generate(&terrain, coord) }));
    }
}

/// Meshes and the shared material, for spawning chunks.
#[derive(SystemParam)]
pub struct Spawner<'w> {
    assets: Res<'w, PropAssets>,
    meshes: ResMut<'w, Assets<Mesh>>,
}

/// Spawns the entities of one finished chunk and returns what is needed to attach its colliders later.
fn spawn_chunk(
    commands: &mut Commands,
    spawner: &mut Spawner,
    coord: IVec2,
    data: ChunkData,
) -> Loaded {
    let center = chunk_center(coord);
    let material = &spawner.assets.material;
    let entity = commands
        .spawn((
            Mesh3d(spawner.meshes.add(data.mesh)),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(center.x, 0.0, center.y),
            RigidBody::Static,
            DespawnOnEnter(Screen::Title),
        ))
        .id();
    let cover = Cover {
        props: data.props.map(|m| spawner.meshes.add(m)),
        grass: data.grass.map(|m| spawner.meshes.add(m)),
    };
    let props = spawn_props(commands, entity, material, cover);
    spawn_structures(
        commands,
        entity,
        &spawner.assets,
        &mut spawner.meshes,
        data.structures,
    );
    Loaded {
        entity,
        props,
        heights: data.heights,
        bodies: data.bodies,
        solid: false,
    }
}

/// Turns the nearest finished chunk tasks into entities, a few per frame once the world is ready.
pub fn collect(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    focus: Focus,
    ready: Res<WorldReady>,
) {
    let Some(center) = focus.chunk() else {
        return;
    };
    let budget = if ready.0 { FRAME_BUDGET } else { MAX_PENDING };
    let finished = chunks
        .pending
        .iter()
        .filter(|(_, task)| task.is_finished())
        .map(|(c, _)| *c)
        .collect();
    for coord in select_batch(finished, center, budget) {
        let Some(data) = chunks
            .pending
            .remove(&coord)
            .and_then(|mut t| block_on(poll_once(&mut t)))
        else {
            continue;
        };
        let mut loaded = spawn_chunk(&mut commands, &mut spawner, coord, data);
        loaded.set_solid(&mut commands, needs_collider(coord, center));
        chunks.loaded.insert(coord, loaded);
    }
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
