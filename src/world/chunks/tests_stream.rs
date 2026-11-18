//! A headless run of the streaming chain: load a planet, cross a face edge, leave for space and
//! come back.

use std::time::{Duration, Instant};

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::cut::visible;
use super::place::{Sited, pose};
use super::{Chunks, FRAME_BUDGET};
use crate::space::Anchor;
use crate::world::focus::Link;
use crate::world::globe::{Chart, Face};
use crate::world::props::PropAssets;
use crate::world::{Planet, SpawnPoint, Terrain, WorldReady};

/// Returns an app with a new world begun and the streaming chain, without a window or a screen.
pub(super) fn world() -> App {
    let mut app = App::new();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut images = Assets::<Image>::default();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PropAssets::new(&mut materials, &mut images))
        .insert_resource(materials)
        .insert_resource(images)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Chunks>();
    let begun = app.world_mut().run_system_once(crate::world::begin_world);
    begun.expect("the world begins");
    app.add_systems(
        Update,
        (
            super::change_planet.run_if(resource_exists_and_changed::<Planet>),
            super::follow_face,
            super::watch_far,
            super::grow,
            super::stream,
            super::collect,
            super::refine,
            super::show,
            super::mark_ready,
            super::place,
        )
            .chain(),
    );
    app
}

/// Moves the focus to planet position `to`, when given, and runs frames until `done` holds. Fails
/// after three minutes.
pub(super) fn go(app: &mut App, to: Option<DVec3>, what: &str, done: impl Fn(&World) -> bool) {
    if let Some(at) = to {
        let anchor = *app.world().resource::<Anchor>();
        let spawn = SpawnPoint(anchor.to_render(at));
        app.world_mut().insert_resource(spawn);
        app.update();
    }
    let start = Instant::now();
    while !done(app.world()) {
        assert!(
            start.elapsed() < Duration::from_secs(180),
            "timed out waiting for {what}"
        );
        app.update();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Returns true when every node has its mesh and no detail chunk is missing or being built. A
/// planet without ground never wants a detail chunk: its quadtree only has to be grown.
pub(super) fn idle(world: &World) -> bool {
    let chunks = world.resource::<Chunks>();
    let whole = chunks.tree.leaves().iter().all(|id| chunks.nodes.has(*id));
    let deck = !world.resource::<Terrain>().globe().has_ground() && chunks.grown.is_some();
    whole && (chunks.settled.is_some() || deck) && chunks.pending.is_empty()
}

/// Asserts that what is drawn covers the globe exactly once and sits where the anchor puts it.
pub(super) fn assert_drawn_once(app: &mut App) {
    let chunks = app.world().resource::<Chunks>();
    let cut = visible(&chunks.tree, &|id| chunks.nodes.has(id));
    let area: f64 = cut.iter().map(|id| 0.25f64.powi(i32::from(id.level))).sum();
    assert_eq!(area, 6.0);
    for id in &cut {
        let detail = chunks.loaded.get(id).is_some_and(|l| l.shown);
        assert_ne!(
            chunks.nodes.live[id].visible, detail,
            "{id:?} is drawn once"
        );
    }
    let drawn = chunks.nodes.live.iter().filter(|(_, node)| node.visible);
    assert!(drawn.clone().all(|(id, _)| cut.contains(id)));
    assert!(
        chunks
            .loaded
            .iter()
            .all(|(id, l)| l.shown == cut.contains(id))
    );
    let anchor = *app.world().resource::<Anchor>();
    let link = Link {
        offset: DVec3::ZERO,
        turn: bevy::math::DQuat::IDENTITY,
    };
    let mut sited = app.world_mut().query::<(&Sited, &Transform)>();
    for (site, transform) in sited.iter(app.world()) {
        assert_eq!(pose(&anchor, &link, site).1, *transform);
    }
}

/// Gets the world ready around the spawn, then lets it settle within the frame budget.
pub(super) fn settle_at_spawn(app: &mut App) {
    go(app, None, "the spawn chunks", |w| {
        w.resource::<WorldReady>().0
    });
    let mut before = app.world().resource::<Chunks>().loaded.len();
    while !idle(app.world()) {
        app.update();
        std::thread::sleep(Duration::from_millis(2));
        let now = app.world().resource::<Chunks>().loaded.len();
        assert!(
            now <= before + FRAME_BUDGET + 9,
            "{} chunks in one frame",
            now - before
        );
        before = now;
    }
    assert_drawn_once(app);
    assert!(before >= 40, "only {before} detail chunks");
}

/// Walks the focus over the edge of its face and returns the direction it ends in.
fn cross_edge(app: &mut App) -> DVec3 {
    let terrain = app.world().resource::<Terrain>().clone();
    let Chart::Face { face, .. } = terrain.chart() else {
        panic!("the terrain is a face chart");
    };
    let [normal, u, _] = face.axes();
    let across = (normal * 0.98 + u).normalize();
    go(
        app,
        Some(across * (terrain.globe().radius() + 3.0)),
        "the chunks across the edge",
        idle,
    );
    let moved = app.world().resource::<Terrain>().chart();
    assert_eq!(moved, terrain.globe().face_chart(Face::locate(across).0));
    assert_ne!(moved, terrain.chart());
    assert!(!app.world().resource::<Chunks>().loaded.is_empty());
    assert_drawn_once(app);
    across
}

/// Lifts the focus to five radii along `dir`, where the baked planet replaces the quadtree, and
/// brings it back to two.
fn leave_and_return(app: &mut App, dir: DVec3) {
    let radius = app.world().resource::<Terrain>().globe().radius();
    go(app, Some(dir * radius * 5.0), "the baked planet", |w| {
        w.resource::<Chunks>().far.active()
    });
    let chunks = app.world().resource::<Chunks>();
    assert!(chunks.loaded.is_empty() && chunks.nodes.live.is_empty() && chunks.tree.count() == 6);
    go(
        app,
        Some(dir * radius * 2.0),
        "the quadtree to come back",
        idle,
    );
    assert!(!app.world().resource::<Chunks>().far.active());
    assert_drawn_once(app);
}

/// The world gets ready around the spawn, keeps to the frame budget, follows the focus over a
/// face edge, swaps to the baked planet in space and streams again on the way back.
#[test]
fn planet_streams_around_a_moving_focus() {
    let mut app = world();
    settle_at_spawn(&mut app);
    let across = cross_edge(&mut app);
    leave_and_return(&mut app, across);
}
