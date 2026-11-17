//! Main thread side of chunk streaming: turns finished chunk data into entities.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::build::{ChunkData, NodeData};
use super::detail::Detail;
use super::nodes::{Coarse, Node};
use super::place::{Sited, pose};
use super::plan::Loaded;
use super::{FRAME_BUDGET, MAX_PENDING};
use crate::render::RayTracing;
use crate::screens::Screen;
use crate::space::Placed;
use crate::world::focus::Frames;
use crate::world::globe::Face;
use crate::world::mesh::Frame;
use crate::world::props::{Cover, PropAssets, spawn_props};
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
    frames: Frames<'w>,
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

    /// Returns the view of the planet through the chart of `face`, the one its chunks are built in.
    pub fn view(&self, face: Face) -> Terrain {
        let globe = self.terrain.globe();
        Terrain::on(globe.clone(), globe.face_chart(face))
    }

    /// Returns the pose components of an entity whose local coordinates are those of `frame`:
    /// on the planet, in the anchor frame and, from the current anchor, in render space.
    pub fn sited(&self, frame: Frame) -> (Sited, Placed, Transform) {
        let sited = Sited::from(frame);
        let fallback = (
            Placed {
                pos: sited.pos,
                rot: sited.rot,
            },
            Transform::default(),
        );
        let link = self.frames.link();
        let (placed, transform) =
            link.map_or(fallback, |(anchor, link)| pose(&anchor, &link, &sited));
        (sited, placed, transform)
    }

    /// Spawns the terrain entity of a detail chunk around `mesh`, stored in `frame`.
    fn terrain(&mut self, commands: &mut Commands, frame: Frame, mesh: Mesh) -> Entity {
        let mesh = self.meshes.add(mesh);
        let material = self.assets.material.clone();
        let bundle = (Mesh3d(mesh), MeshMaterial3d(material), self.sited(frame));
        commands
            .spawn((bundle, RigidBody::Static, DespawnOnEnter(Screen::Title)))
            .id()
    }

    /// Spawns the hidden entity of a coarse node, or of the far planet, around its mesh.
    pub fn coarse(&mut self, commands: &mut Commands, data: NodeData) -> Node {
        let mesh = self.meshes.add(data.mesh);
        let material = MeshMaterial3d(self.assets.material.clone());
        let bundle = (Mesh3d(mesh.clone()), material, self.sited(data.frame));
        let hidden = (Coarse, Visibility::Hidden, DespawnOnEnter(Screen::Title));
        Node {
            entity: commands.spawn((bundle, hidden)).id(),
            mesh,
            visible: false,
        }
    }
}

/// Spawns the entities of one finished chunk and returns what is needed to attach its colliders later.
pub fn spawn_chunk(commands: &mut Commands, spawner: &mut Spawner, data: ChunkData) -> Loaded {
    let entity = spawner.terrain(commands, data.frame, data.mesh);
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
        shown: true,
        detail: Detail::new(data.lod, children),
    }
}
