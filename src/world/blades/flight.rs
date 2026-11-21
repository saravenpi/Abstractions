//! Bands of tiles of blades built off the main thread: a task per band on a pool of their own, landed a few per
//! frame nearest first, so a camera that walks onto new ground never waits for the blades under it, nor for the
//! chunks and nodes that fill the shared compute pool.

use std::sync::{Arc, OnceLock};

use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::NoAutoAabb;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder, block_on, futures_lite::future::poll_once};

use super::Cropped;
use super::clear::{blocked, footprints, reaching};
use super::fade::ranks;
use super::ground::Ground;
use super::tile::grow;
use super::tiles::{Grower, Spot, Tile, chunk_of};
use super::tune::{FLIGHTS, LANDS, TILE, WORKERS};
use crate::screens::Screen;
use crate::space::Placed;
use crate::world::Terrain;
use crate::world::chunks::{Sited, pose};
use crate::world::mesh::Frame;
use crate::world::scatter::Carpet;

/// What a band builds: its mesh, the box it is culled by and its frame, or nothing where no blade grows.
type Drawn = Option<(Mesh, Aabb, Frame)>;

/// A band of a tile being built on the pool of the blades.
pub struct Flight {
    /// The task that builds the mesh.
    task: Task<Drawn>,
}

/// Returns the mesh of the blades of band `band` of tile `key` of `terrain` over `carpet`, clear of the
/// footprints `near` that reach the tile and of the patches `cropped`, its box and the frame it is built in.
fn blades_of(
    terrain: &Terrain,
    carpet: &Carpet,
    (key, band): Spot,
    (near, cropped): (&[(DVec3, f32)], &Cropped),
) -> Drawn {
    let ground = Ground::read(terrain, key);
    let near = reaching(near, ground.sphere(terrain));
    let clear = |p| blocked(p, (&near, cropped), terrain.globe());
    let (blades, frame) = grow((terrain, &ground), key, carpet, ranks(band), &clear);
    let bounds = blades.bounds();
    Some((blades.into_mesh()?, bounds, frame))
}

/// The footprints that clear blades and the cropped patches, as positions with radii.
pub type Around = (Vec<(DVec3, f32)>, Vec<(DVec3, f32)>);

impl Flight {
    /// Starts building band `at` of its tile over `carpet`, clear of the footprints and cropped patches in
    /// `around`, on the pool of the blades: [`WORKERS`] threads of its own, made on first use.
    pub fn start(terrain: &Terrain, carpet: Arc<Carpet>, at: Spot, around: Around) -> Self {
        static POOL: OnceLock<TaskPool> = OnceLock::new();
        let pool = POOL.get_or_init(|| {
            TaskPoolBuilder::new()
                .num_threads(WORKERS)
                .thread_name("Blades".to_string())
                .build()
        });
        let terrain = terrain.clone();
        let (near, cropped) = (around.0, Cropped(around.1));
        let task = pool
            .spawn(async move { blades_of(&terrain, &carpet, at, (near.as_slice(), &cropped)) });
        Self { task }
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

/// Spawns the entity that draws `mesh` in `frame`, culled by `bounds`, posed from the anchor of now.
fn spawn_tile(g: &mut Grower, mesh: Mesh, bounds: Aabb, frame: Frame) -> Entity {
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
        bounds,
        NoAutoAabb,
    );
    let at = (sited, placed, transform, Visibility::default());
    let name = (Name::new("Blades"), DespawnOnEnter(Screen::Title));
    g.commands.spawn((name, look, at)).id()
}

/// Puts `drawn` in place of whatever band `spot` held: despawns its entity, spawns the new one.
fn settle(g: &mut Grower, spot: Spot, drawn: Drawn) {
    if let Some(entity) = g.meadow.tiles.remove(&spot).and_then(|t| t.entity) {
        g.commands.entity(entity).try_despawn();
    }
    let entity = drawn.map(|(mesh, bounds, frame)| spawn_tile(g, mesh, bounds, frame));
    let tile = Tile {
        entity,
        stale: false,
    };
    g.meadow.tiles.insert(spot, tile);
}

/// Lands the bands that finished building, at most [`LANDS`] a frame.
pub fn land(g: &mut Grower) {
    for _ in 0..LANDS {
        let done = g.meadow.flights.iter().find(|(_, f)| f.finished());
        let Some(spot) = done.map(|(spot, _)| *spot) else {
            return;
        };
        if let Some(flight) = g.meadow.flights.remove(&spot) {
            settle(g, spot, flight.drawn());
        }
    }
}

/// Starts building the nearest of the `wanted` bands, at most [`LANDS`] a frame and [`FLIGHTS`] at once.
/// A band where no ground cover grows is settled bare at once.
pub fn fly(g: &mut Grower, wanted: &[(f32, Spot)], near: &mut Vec<(DVec3, f32)>) {
    let mut starts = 0;
    for &(_, spot) in wanted {
        let key = spot.0;
        if starts >= LANDS || g.meadow.flights.len() >= FLIGHTS {
            break;
        }
        if g.meadow.flights.contains_key(&spot) {
            continue;
        }
        let Some(carpet) = g.meadow.carpet(&g.terrain, chunk_of(key)) else {
            settle(g, spot, None);
            starts += 1;
            continue;
        };
        let centre = (key.as_vec2() + 0.5) * TILE;
        footprints(&g.chunks, g.terrain.embed(centre.x, 0.0, centre.y), near);
        let around = (near.clone(), g.cropped.0.clone());
        let flight = Flight::start(&g.terrain, carpet, spot, around);
        g.meadow.flights.insert(spot, flight);
        starts += 1;
    }
}
