//! Streaming bodies other than a planet of the system: a small moon, which streams like any planet,
//! and a gas giant, whose smooth deck is coarse nodes alone.

use avian3d::prelude::Collider;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::reach::STEPS;
use super::tests_reach::{resize, settle_on};
use super::tests_stream::{assert_drawn_once, go, idle, settle_at_spawn, world};
use super::{Chunks, rehome};
use crate::space::{Anchor, Location, Travel};
use crate::universe::{BodyId, Orbit, PlanetClass, PlanetRecord, PlanetSpec, Universe};
use crate::world::enter::planet_of;
use crate::world::focus::Link;
use crate::world::{SpawnPoint, WorldReady};

/// Returns a hand-built body of `radius` metres and `class`, from the home planet of seed 4: the first
/// moon of the fifth planet when `moon` is set, the planet itself else.
pub(super) fn body(radius: f64, class: PlanetClass, moon: bool) -> PlanetSpec {
    let universe = Universe::new(4);
    let home = universe.planet(universe.home());
    let star = home.orbit();
    let around = Orbit {
        radius: 4.0e5,
        seconds: 3.0e5,
        phase: 1.1,
    };
    let own = [star, around][usize::from(moon)];
    PlanetSpec {
        id: BodyId {
            index: 4,
            moon: u8::from(moon),
            ..home.id
        },
        radius,
        gravity: 1.6,
        day_seconds: 3.0e5,
        orbit_radius: own.radius,
        orbit_seconds: own.seconds,
        orbit_phase: own.phase,
        record: PlanetRecord {
            class,
            ocean: 0.0,
            ..PlanetRecord::TEMPERATE
        },
        host: moon.then_some(star),
        ..home
    }
}

/// Makes `spec` the streamed body, with the focus `lift` metres over the north pole, or at the
/// terrain spawn without one.
pub(super) fn enter(app: &mut App, spec: &PlanetSpec, lift: Option<f32>) {
    let (planet, north) = planet_of(spec.id, spec);
    let at = lift.map_or_else(|| north.find_spawn(), |lift| Vec3::Y * lift);
    let (terrain, spawn) = rehome(&north, at);
    let world = app.world_mut();
    world.insert_resource(SpawnPoint(spawn));
    world.insert_resource(Anchor::surface(spec.id, &terrain, spawn));
    world.insert_resource(Location {
        body: spec.id,
        travel: Travel::OnFoot,
    });
    world.insert_resource(terrain);
    world.insert_resource(planet);
}

/// A 5 km moon gets ready around its spawn and settles within the frame budget, drawn once with
/// its detail chunks, as a planet does.
#[test]
fn a_moon_streams_like_a_planet() {
    let mut app = world();
    let moon = body(5000.0, PlanetClass::Barren, true);
    enter(&mut app, &moon, None);
    settle_at_spawn(&mut app);
}

/// A moon sits where its planet is plus its own place around it. A spin that turns the way its orbit
/// goes keeps one face to its planet; `spin` with a day as long as the orbit turns the other way, and
/// so swings that face twice round its sky per orbit.
#[test]
fn a_moon_follows_its_planet_and_its_spin_sets_its_near_side() {
    let moon = body(5000.0, PlanetClass::Barren, true);
    let host = moon.host.expect("a moon has a host");
    for t in [0.0, 1.0e4, 7.7e4, 2.0e5] {
        let offset = Link::orbiting(&moon, t).offset;
        assert!((offset - host.at(t) - moon.orbit().at(t)).length() < 1e-3);
    }
    let near =
        |spec: &PlanetSpec, t: f64| Link::orbiting(spec, t).to_planet(host.at(t)).normalize();
    let locked = PlanetSpec {
        day_seconds: -moon.orbit_seconds,
        ..moon.clone()
    };
    let quarter = moon.orbit_seconds / 4.0;
    assert!(near(&locked, 0.0).distance(near(&locked, quarter)) < 1e-6);
    assert_eq!(moon.day_seconds, moon.orbit_seconds);
    assert!(near(&moon, 0.0).dot(near(&moon, quarter)) < -0.99);
}

/// A 150 km gas giant settles as coarse nodes alone, drawn once over the whole deck, ready at once,
/// with no detail chunk, no collider and no node below its deepest level, and bakes its far sphere.
#[test]
fn a_giant_streams_only_its_deck() {
    let mut app = world();
    let giant = body(150_000.0, PlanetClass::GasGiant, false);
    enter(&mut app, &giant, Some(1500.0));
    go(&mut app, None, "the deck to settle", idle);
    assert!(app.world().resource::<WorldReady>().0);
    assert_drawn_once(&mut app);
    let chunks = app.world().resource::<Chunks>();
    let deepest = chunks.tree.leaves().iter().map(|id| id.level).max();
    let globe = app
        .world()
        .resource::<crate::world::Terrain>()
        .globe()
        .clone();
    assert_eq!(deepest, Some(globe.deepest_level()));
    assert!(chunks.loaded.is_empty() && chunks.pending.is_empty());
    let colliders = app
        .world_mut()
        .query::<&Collider>()
        .iter(app.world())
        .count();
    assert_eq!(colliders, 0);
    let far = DVec3::Y * globe.radius() * 5.0;
    go(&mut app, Some(far), "the baked deck", |w| {
        w.resource::<Chunks>().far.active()
    });
}

/// Prints what a 150 km gas giant costs at Medium and High against a 40 km planet, and checks that it
/// holds fewer nodes and triangles. Run it alone:
/// `cargo test --features dev deck_table -- --ignored --nocapture`.
#[test]
#[ignore = "a benchmark of a few minutes; run it alone"]
fn deck_table() {
    let giant = body(150_000.0, PlanetClass::GasGiant, false);
    for (name, rings) in STEPS.into_iter().filter(|(_, r)| [4, 8].contains(r)) {
        let deck = settle_on(rings, &|app| enter(app, &giant, Some(1500.0)));
        let planet = settle_on(rings, &|app| resize(app, 40_000.0));
        for (what, load) in [("150 km giant", &deck), ("40 km planet", &planet)] {
            println!(
                "deck {what} {name:>6}: {} nodes, {} chunks, {} triangles, {} colliders, \
                 {} frames, {:.2} ms mean and {:.1} ms worst on the main thread",
                load.nodes,
                load.chunks,
                load.triangles,
                load.colliders,
                load.frames,
                load.main.0 / f64::from(load.frames) * 1e3,
                load.main.1 * 1e3
            );
        }
        assert!(deck.nodes <= planet.nodes && deck.triangles <= planet.triangles);
        assert_eq!((deck.chunks, deck.colliders), (0, 0));
    }
}
