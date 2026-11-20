//! What each render distance step costs once settled, on the home planet and on a 40 km planet:
//! quadtree nodes, detail chunks, triangles drawn, colliders and mesh megabytes.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use avian3d::prelude::Collider;
use bevy::prelude::*;

use super::reach::{Reach, STEPS};
use super::tests_stream::{idle, world};
use super::{Chunks, rehome};
use crate::space::{Anchor, Location};
use crate::world::globe::Face;
use crate::world::{Globe, Planet, SpawnPoint, Terrain};

/// What a settled world holds.
#[derive(Default)]
pub(super) struct Load {
    /// Quadtree nodes, hidden parents included.
    pub(super) nodes: usize,
    /// Detail chunks.
    pub(super) chunks: usize,
    /// Triangles drawn.
    pub(super) triangles: usize,
    /// Colliders.
    pub(super) colliders: usize,
    /// Megabytes of meshes.
    megabytes: f64,
    /// Seconds until settled.
    pub(super) seconds: f64,
    /// Frames run until settled.
    pub(super) frames: u32,
    /// Seconds the frames took on the main thread, together and the longest one.
    pub(super) main: (f64, f64),
}

/// Swaps the streamed planet for one of `radius` metres, with its spawn and anchor.
pub(super) fn resize(app: &mut App, radius: f64) {
    let globe = Arc::new(Globe::new(7, radius, 9.81));
    let north = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    let (terrain, spawn) = rehome(&north, north.find_spawn());
    let body = app.world().resource::<Location>().body;
    let world = app.world_mut();
    world.insert_resource(SpawnPoint(spawn));
    world.insert_resource(Anchor::surface(body, &terrain, spawn));
    world.insert_resource(terrain);
    world.insert_resource(Planet {
        body,
        globe,
        day_seconds: 1200.0,
    });
}

/// Returns true when `entity` and every parent of it are shown.
fn shown(world: &World, entity: Entity) -> bool {
    let hidden = world.get::<Visibility>(entity) == Some(&Visibility::Hidden);
    let parent = world.get::<ChildOf>(entity).map(|c| c.parent());
    !hidden && parent.is_none_or(|p| shown(world, p))
}

/// Returns the triangles and the bytes of vertices and indices of the mesh `handle`.
fn size(meshes: &Assets<Mesh>, handle: &Handle<Mesh>) -> (usize, usize) {
    meshes.get(handle).map_or((0, 0), |m| {
        let indices = m.indices().map_or(0, |i| i.len());
        (
            indices / 3,
            m.get_vertex_size() as usize * m.count_vertices() + indices * 4,
        )
    })
}

/// Measures what the settled world holds.
fn measure(app: &mut App, seconds: f64) -> Load {
    let world = app.world_mut();
    let mut drawn = world.query::<(Entity, &Mesh3d)>();
    let parts: Vec<(Entity, Handle<Mesh>)> =
        drawn.iter(world).map(|(e, m)| (e, m.0.clone())).collect();
    let colliders = world.query::<&Collider>().iter(world).count();
    let meshes = world.resource::<Assets<Mesh>>();
    let shown_parts = parts.iter().filter(|(e, _)| shown(world, *e));
    let triangles = shown_parts.map(|(_, h)| size(meshes, h).0).sum();
    let mut seen = HashSet::new();
    let unique = parts.iter().filter(|(_, h)| seen.insert(h.id()));
    let megabytes = unique.map(|(_, h)| size(meshes, h).1).sum::<usize>();
    let chunks = world.resource::<Chunks>();
    Load {
        nodes: chunks.tree.count(),
        chunks: chunks.loaded.len(),
        triangles,
        colliders,
        megabytes: megabytes as f64 / 1.0e6,
        seconds,
        ..Load::default()
    }
}

/// Settles a world of `radius` metres at `rings`, waits for the last fade to end and returns
/// what it holds.
pub(super) fn settle(radius: f64, rings: u32) -> Load {
    settle_on(rings, &|app| resize(app, radius))
}

/// Settles a world whose streamed body `enter` sets, at `rings`, waits for the last fade to end
/// and returns what it holds, with the time its frames took on the main thread.
pub(super) fn settle_on(rings: u32, enter: &dyn Fn(&mut App)) -> Load {
    let mut app = world();
    app.insert_resource(Reach::of(Some(rings)));
    enter(&mut app);
    let (start, mut frames, mut main) = (Instant::now(), 0, (0.0, 0.0f64));
    while frames == 0 || !(idle(app.world()) && app.world().resource::<Chunks>().fades.idle()) {
        assert!(
            start.elapsed() < Duration::from_secs(900),
            "{rings} rings never settled"
        );
        let frame = Instant::now();
        app.update();
        let took = frame.elapsed().as_secs_f64();
        (frames, main) = (frames + 1, (main.0 + took, main.1.max(took)));
        std::thread::sleep(Duration::from_millis(1));
    }
    let load = measure(&mut app, start.elapsed().as_secs_f64());
    Load {
        frames,
        main,
        ..load
    }
}

/// Radius of Earth in metres, whose quadtree is 17 levels deep.
pub(super) const EARTH: f64 = 6_371_000.0;

/// Prints what every render distance step costs on a 6 km, a 40 km and an Earth-sized planet, and checks that the
/// quadtree keeps within its cap. Run it alone: `cargo test --features dev reach_table -- --ignored --nocapture`.
#[test]
#[ignore = "a benchmark of several minutes; run it alone"]
fn reach_table() {
    for radius in [6_000.0, 40_000.0, EARTH] {
        let depth = Globe::new(7, radius, 9.81).deepest_level();
        for (name, rings) in STEPS {
            let load = settle(radius, rings);
            println!(
                "reach {:>3} km {name:>8} ({rings:>2} rings): {} nodes, {} chunks, {} triangles drawn, \
                 {} colliders, {:.0} MB of meshes, settled in {:.0} s",
                radius / 1000.0,
                load.nodes,
                load.chunks,
                load.triangles,
                load.colliders,
                load.megabytes,
                load.seconds
            );
            assert!(load.nodes <= Reach::of(Some(rings)).node_cap(depth) + 4);
        }
    }
}
