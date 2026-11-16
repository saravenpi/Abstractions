//! Main thread side of chunk streaming: turns finished chunk data into entities.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::build::ChunkData;
use super::detail::Detail;
use super::plan::Loaded;
use super::{FRAME_BUDGET, MAX_PENDING};
use crate::render::RayTracing;
use crate::screens::Screen;
use crate::world::props::{Cover, PropAssets, spawn_props};
use crate::world::scatter::chunk_center;
use crate::world::structures::spawn_structures;
use crate::world::{Terrain, WorldReady};

/// Meshes, the shared material, terrain and ray tracing state, for spawning and refining chunks.
#[derive(SystemParam)]
pub struct Spawner<'w> {
    pub(super) assets: Res<'w, PropAssets>,
    pub(super) meshes: ResMut<'w, Assets<Mesh>>,
    pub(super) ray_tracing: Res<'w, RayTracing>,
    pub(super) terrain: Res<'w, Terrain>,
    pub(super) ready: Res<'w, WorldReady>,
}

impl Spawner<'_> {
    /// Returns how many finished chunks may become entities this frame.
    pub fn budget(&self) -> usize {
        if self.ready.0 {
            FRAME_BUDGET
        } else {
            MAX_PENDING
        }
    }

    /// Spawns the terrain entity of chunk `coord` around `mesh`.
    fn terrain(&mut self, commands: &mut Commands, coord: IVec2, mesh: Mesh) -> Entity {
        let center = chunk_center(coord);
        let mesh = self.meshes.add(mesh);
        let material = self.assets.material.clone();
        let transform = Transform::from_xyz(center.x, 0.0, center.y);
        let bundle = (Mesh3d(mesh), MeshMaterial3d(material), transform);
        commands
            .spawn((bundle, RigidBody::Static, DespawnOnEnter(Screen::Title)))
            .id()
    }
}

/// Spawns the entities of one finished chunk and returns what is needed to attach its colliders later.
pub fn spawn_chunk(
    commands: &mut Commands,
    spawner: &mut Spawner,
    coord: IVec2,
    data: ChunkData,
) -> Loaded {
    let entity = spawner.terrain(commands, coord, data.mesh);
    let material = spawner.assets.material.clone();
    let children = data.props.is_some() || data.grass.is_some() || data.structures.is_some();
    let cover = Cover {
        props: data.props.map(|m| spawner.meshes.add(m)),
        grass: data.grass.map(|m| spawner.meshes.add(m)),
    };
    let props = spawn_props(commands, entity, &material, cover);
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
        detail: Detail::new(data.lod, children),
    }
}
