//! Climbing to orbit: the ship goes straight up from the ground of the home-sized planet to
//! above its air, by steps as the headless tour moves it and at steady rates, while the fades run.
//! What is drawn must cover the globe at every frame, and exactly once when nothing fades.
//!
//! Bevy picks the dithered pipeline of a mesh by whether it has a visibility range when it is
//! first drawn, and picks again only when its mesh or its material changes. A mesh drawn with the
//! dithered pipeline and no range discards every pixel, so each fade that adds or removes a range
//! must mark the mesh changed: the orbit hole of the 0.8.0 tour was merged nodes that stayed
//! invisible once their fade ended.

use std::collections::HashMap;

use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

use super::super::Chunks;
use super::super::cut::visible;
use super::super::dither::Way;
use super::{Route, frame, rest, stage};
use crate::world::globe::ChunkId;

/// Heights over the sea in metres the headless tour puts the ship at on its way to orbit.
const TOUR: [f64; 7] = [30.0, 100.0, 300.0, 500.0, 1000.0, 1400.0, 1700.0];

/// Frames the tour holds each height before its shot.
const HOLD: usize = 40;

/// Height of the orbit in metres over the sea: under the 1.3 radii at which the baked sphere takes over
/// from the quadtree, 1,800 m over this planet.
const ORBIT: f64 = 1700.0;

/// Meshes that gained or lost a visibility range without being marked changed: Bevy kept the
/// pipeline it chose before for them.
#[derive(Resource, Default)]
struct Stale(Vec<Entity>);

/// Notes, at the end of every frame, each mesh whose visibility range came or went while its
/// mesh was left unchanged.
fn watch(
    meshes: Query<(Entity, Ref<Mesh3d>, Has<VisibilityRange>)>,
    mut ranged: Local<HashMap<Entity, bool>>,
    mut stale: ResMut<Stale>,
) {
    for (entity, mesh, now) in &meshes {
        let before = ranged.insert(entity, now);
        if before.is_some_and(|was| was != now) && !mesh.is_changed() {
            stale.0.push(entity);
        }
    }
}

/// Returns true when `entity` or anything under it carries a visibility range.
fn ranged(world: &World, entity: Entity) -> bool {
    let below = world.get::<Children>(entity).into_iter().flatten();
    world.get::<VisibilityRange>(entity).is_some() || below.clone().any(|kid| ranged(world, *kid))
}

/// Returns true when `entity` is not hidden.
fn shows(world: &World, entity: Entity) -> bool {
    world
        .get::<Visibility>(entity)
        .is_some_and(|v| *v != Visibility::Hidden)
}

/// Returns the entity that draws node `id` of the cut: its shown detail chunk, else its node.
fn drawer(chunks: &Chunks, id: ChunkId) -> Entity {
    let shown = chunks.loaded.get(&id).filter(|l| l.shown);
    shown.map_or_else(|| chunks.nodes.live[&id].entity, |l| l.entity)
}

/// Asserts that every node of the draw cut is drawn by an entity that is not hidden, and that
/// it is whole or fading in. With `settled`, nothing may fade at all, and no node outside the
/// cut, nor one under a shown chunk, may be drawn.
fn assert_covered(app: &App, settled: bool, when: &str) {
    let (world, chunks) = (app.world(), app.world().resource::<Chunks>());
    let cut = visible(&chunks.tree, &|id| chunks.nodes.has(id));
    let area: f64 = cut.iter().map(|id| 0.25f64.powi(i32::from(id.level))).sum();
    assert_eq!(area, 6.0, "{when}: the cut covers the globe");
    for id in &cut {
        let entity = drawer(chunks, *id);
        assert!(shows(world, entity), "{when}: {id:?} is hidden");
        let fading = chunks.fades.way(entity) == Some(Way::In) && !settled;
        assert!(
            fading || !ranged(world, entity),
            "{when}: {id:?} keeps a range"
        );
    }
    if settled {
        for (id, node) in &chunks.nodes.live {
            let under = !cut.contains(id) || chunks.loaded.get(id).is_some_and(|l| l.shown);
            let drawn = shows(world, node.entity) || ranged(world, node.entity);
            assert!(!(under && drawn), "{when}: {id:?} is drawn twice");
        }
    }
}

/// Holds `route` for `frames` frames at the pace of play, checking the cover at every one.
fn hold_for(app: &mut App, route: &mut Route, frames: usize) {
    for at in 0..frames {
        frame(app, route);
        let when = format!("{} m, frame {at}", route.height);
        let stale = &app.world().resource::<Stale>().0;
        assert!(stale.is_empty(), "{when}: {stale:?} keep a stale pipeline");
        assert_covered(app, false, &when);
    }
}

/// The ship that jumps from height to height as the tour moves it, and one that climbs
/// steadily, slowly and fast, reach orbit with the globe drawn whole at every frame, exactly
/// once when everything has settled, and no mesh left with the pipeline of a fade it left.
#[test]
fn climbing_to_orbit_keeps_the_globe_whole() {
    let (mut app, mut route) = stage(6000.0, 4);
    app.init_resource::<Stale>().add_systems(Last, watch);
    for height in TOUR {
        route.height = height;
        hold_for(&mut app, &mut route, HOLD);
    }
    rest(&mut app, &route);
    assert_covered(&app, true, "orbit after the tour");
    for rate in [60.0, 400.0] {
        route.height = 30.0;
        rest(&mut app, &route);
        let frames = ((ORBIT - route.height) / rate * 60.0) as usize;
        for _ in 0..frames {
            route.height += rate / 60.0;
            hold_for(&mut app, &mut route, 1);
        }
        rest(&mut app, &route);
        assert_covered(&app, true, &format!("orbit at {rate} m/s"));
    }
    assert!(app.world().resource::<Stale>().0.is_empty());
}

/// Heights over the sea in metres the tour puts the ship at over an Earth-sized planet, up to
/// where the world still draws the quadtree, under the 1.3 radii of the baked sphere.
const EARTH_TOUR: [f64; 6] = [30.0, 300.0, 3.0e3, 3.0e4, 3.0e5, 1.8e6];

/// Over a planet of 6,371 km, 17 levels deep, the ship that jumps from height to height keeps
/// the globe drawn whole at every frame, exactly once when everything has settled, and leaves
/// no mesh with the pipeline of a fade it left. The world settles after each jump in the five
/// minutes `rest` allows.
#[test]
fn climbing_over_an_earth_keeps_the_globe_whole() {
    let (mut app, mut route) = stage(6.371e6, 4);
    app.init_resource::<Stale>().add_systems(Last, watch);
    for height in EARTH_TOUR {
        route.height = height;
        hold_for(&mut app, &mut route, HOLD);
        rest(&mut app, &route);
        assert_covered(&app, true, &format!("{height} m over Earth"));
    }
    assert!(app.world().resource::<Stale>().0.is_empty());
}
