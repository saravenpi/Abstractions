//! Tiles of blades built off the main thread: a task per tile on the compute pool, landed a few per frame
//! nearest first, so a camera that walks onto new ground never waits for the blades under it.

use std::sync::Arc;

use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::Cropped;
use super::clear::{footprints, reaching, taken};
use super::ground::Ground;
use super::tile::grow;
use super::tiles::{Grower, Tile, chunk_of};
use super::tune::{FLIGHTS, LANDS, TILE};
use crate::screens::Screen;
use crate::space::Placed;
use crate::world::Terrain;
use crate::world::chunks::{Sited, pose};
use crate::world::mesh::Frame;
use crate::world::scatter::Carpet;

/// What a tile builds: its mesh and frame, or nothing where no blade grows.
type Drawn = Option<(Mesh, Frame)>;

/// A tile being built on the compute pool.
pub struct Flight {
    /// The task that builds the mesh.
    task: Task<Drawn>,
    /// True when it builds the dense blades.
    pub full: bool,
}

/// Returns the mesh of the blades of tile `key` of `terrain`, dense when `full`, over `carpet`, clear of the
/// footprints `near` that reach the tile and of the patches `cropped`, and the frame it is built in.
fn blades_of(
    terrain: &Terrain,
    carpet: &Carpet,
    (key, full): (IVec2, bool),
    (near, cropped): (&[(DVec3, f32)], &Cropped),
) -> Drawn {
    let ground = Ground::read(terrain, key);
    let near = reaching(near, ground.sphere(terrain));
    let radius = terrain.globe().radius();
    let clear = |p| taken(p, &near, cropped, radius);
    let (blades, frame) = grow((terrain, &ground), key, carpet, full, &clear);
    Some((blades.into_mesh()?, frame))
}

/// The footprints that clear blades and the cropped patches, as positions with radii.
pub type Around = (Vec<(DVec3, f32)>, Vec<(DVec3, f32)>);

impl Flight {
    /// Starts building tile `key`, dense when `full`, over `carpet`, clear of the footprints and cropped
    /// patches in `around`.
    pub fn start(
        terrain: &Terrain,
        carpet: Arc<Carpet>,
        at: (IVec2, bool),
        around: Around,
    ) -> Self {
        let terrain = terrain.clone();
        let (near, cropped) = (around.0, Cropped(around.1));
        let task = AsyncComputeTaskPool::get()
            .spawn(async move { blades_of(&terrain, &carpet, at, (near.as_slice(), &cropped)) });
        Self { task, full: at.1 }
    }

    /// Returns true once the task has built its mesh.
    pub fn finished(&self) -> bool {
        self.task.is_finished()
    }

    /// Returns what the finished task built.
    fn drawn(mut self) -> Drawn {
        block_on(poll_once(&mut self.task)).flatten()
    }
}

/// Spawns the entity that draws `mesh` in `frame`, posed from the anchor of now.
fn spawn_tile(g: &mut Grower, mesh: Mesh, frame: Frame) -> Entity {
    let sited = Sited::from(frame);
    let alone = Placed {
        pos: sited.pos,
        rot: sited.rot,
    };
    let link = g.frames.link();
    let (placed, transform) = link.map_or((alone, Transform::default()), |(anchor, link)| {
        pose(&anchor, &link, &sited)
    });
    let look = (
        Mesh3d(g.meshes.add(mesh)),
        MeshMaterial3d(g.assets.material.clone()),
        NotShadowCaster,
    );
    let at = (sited, placed, transform, Visibility::default());
    let name = (Name::new("Blades"), DespawnOnEnter(Screen::Title));
    g.commands.spawn((name, look, at)).id()
}

/// Puts `drawn` in place of whatever tile `key` held: despawns its entity, spawns the new one.
fn settle(g: &mut Grower, key: IVec2, full: bool, drawn: Drawn) {
    if let Some(entity) = g.meadow.tiles.remove(&key).and_then(|t| t.entity) {
        g.commands.entity(entity).try_despawn();
    }
    let entity = drawn.map(|(mesh, frame)| spawn_tile(g, mesh, frame));
    let tile = Tile {
        entity,
        full,
        stale: false,
    };
    g.meadow.tiles.insert(key, tile);
}

/// Lands the tiles that finished building, at most [`LANDS`] a frame.
pub fn land(g: &mut Grower) {
    for _ in 0..LANDS {
        let done = g.meadow.flights.iter().find(|(_, f)| f.finished());
        let Some(key) = done.map(|(key, _)| *key) else {
            return;
        };
        if let Some(flight) = g.meadow.flights.remove(&key) {
            let full = flight.full;
            settle(g, key, full, flight.drawn());
        }
    }
}

/// Starts building the nearest of the `wanted` tiles, at most [`LANDS`] a frame and [`FLIGHTS`] at once.
/// A tile where no ground cover grows is settled bare at once.
pub fn fly(g: &mut Grower, wanted: &[(f32, IVec2, bool)], near: &mut Vec<(DVec3, f32)>) {
    let mut starts = 0;
    for &(_, key, full) in wanted {
        if starts >= LANDS || g.meadow.flights.len() >= FLIGHTS {
            break;
        }
        if g.meadow.flights.contains_key(&key) {
            continue;
        }
        let Some(carpet) = g.meadow.carpet(&g.terrain, chunk_of(key)) else {
            settle(g, key, full, None);
            starts += 1;
            continue;
        };
        let centre = (key.as_vec2() + 0.5) * TILE;
        footprints(&g.chunks, g.terrain.embed(centre.x, 0.0, centre.y), near);
        let around = (near.clone(), g.cropped.0.clone());
        let flight = Flight::start(&g.terrain, carpet, (key, full), around);
        g.meadow.flights.insert(key, flight);
        starts += 1;
    }
}
