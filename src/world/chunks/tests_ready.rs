//! Tests of the readiness of the world: a focus in the air gets ready on coarse ground alone,
//! and a focus on the ground still waits for the solid chunks under it.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::near::Around;
use super::tests_stream::{go, world};
use crate::space::Anchor;
use crate::world::{SpawnPoint, Terrain, WorldReady};

/// Returns the planet position of the spawn point of `app`.
fn spawn(app: &App) -> DVec3 {
    let anchor = *app.world().resource::<Anchor>();
    anchor.to_frame(app.world().resource::<SpawnPoint>().0)
}

/// Returns true when the world of `world` is flagged ready.
fn ready(world: &World) -> bool {
    world.resource::<WorldReady>().0
}

/// A body that starts 6560 m up, as the owner's saved place was, gets ready: the quadtree
/// settles on coarse ground under it and no detail chunk is waited for. Before the fix the
/// loading screen stayed on "Generating" for ever here.
#[test]
fn a_focus_in_the_air_gets_ready() {
    let mut app = world();
    let ground = spawn(&app);
    let high = ground + ground.normalize() * 6560.0;
    go(&mut app, Some(high), "a place in the air", ready);
    let chunks = app.world().resource::<Chunks>();
    let terrain = app.world().resource::<Terrain>();
    let around = Around::new(terrain, high).expect("a face chart");
    assert!(chunks.grown.is_some());
    assert!(around.near.iter().all(|id| !chunks.loaded.contains_key(id)));
    assert!(chunks.tree.leaves().iter().all(|id| chunks.nodes.has(*id)));
}

/// A body that starts on the ground is ready only once the leaf under it and the leaves that
/// touch it are solid: the rule for the air never lets the game start over missing ground.
#[test]
fn a_focus_on_the_ground_waits_for_solid_chunks() {
    let mut app = world();
    let ground = spawn(&app);
    go(&mut app, None, "the spawn chunks", ready);
    let chunks = app.world().resource::<Chunks>();
    let terrain = app.world().resource::<Terrain>();
    let around = Around::new(terrain, ground).expect("a face chart");
    let solid = |id| chunks.loaded.get(id).is_some_and(|loaded| loaded.solid);
    assert!(around.near.iter().all(solid));
}

/// A focus a few hundred metres up, where only some of the leaves under it are still built,
/// gets ready too, and every leaf the quadtree holds under it is solid by then.
#[test]
fn a_focus_at_the_edge_of_the_detail_gets_ready() {
    for lift in [400.0, 2068.0] {
        let mut app = world();
        let ground = spawn(&app);
        let high = ground + ground.normalize() * lift;
        go(&mut app, Some(high), "a place low in the air", ready);
        let chunks = app.world().resource::<Chunks>();
        let held = |id: &&crate::world::globe::ChunkId| chunks.tree.leaves().contains(*id);
        let terrain = app.world().resource::<Terrain>();
        let around = Around::new(terrain, high).expect("a face chart");
        let solid = |id| chunks.loaded.get(id).is_some_and(|loaded| loaded.solid);
        assert!(around.near.iter().filter(held).all(solid), "{lift}");
    }
}
