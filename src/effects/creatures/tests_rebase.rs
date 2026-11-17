//! Tests of the pools against the frame systems: a rebase leaves every creature where it was.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::birds::Bird;
use super::butterflies::Butterfly;
use super::fireflies::Firefly;
use super::flock::Flocks;
use super::wander::Wander;
use super::{carry, gear};
use crate::effects::pool::{self, POOL_SIZE, Particle};
use crate::player::Player;
use crate::space::{Anchor, Location, Rebased, Travel};
use crate::universe::Universe;
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Filter of the pooled bodies: every free effect entity but the player.
type Pooled = (With<Rebased>, Without<Player>, Without<ChildOf>);

/// Returns an app on a 3 km planet with the frame systems, the carry system, a player body
/// far enough from the anchored spot for a rebase, and every effect pool spawned.
fn app() -> App {
    let globe = Arc::new(Globe::new(4, 3000.0, 22.0));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosX)).around(40.0, -25.0);
    let body = Universe::new(4).home();
    let mut app = App::new();
    crate::space::plugin(&mut app);
    app.insert_resource(Anchor::surface(body, &terrain, Vec3::new(40.0, 2.0, -25.0)))
        .insert_resource(terrain)
        .insert_resource(Location {
            body,
            travel: Travel::OnFoot,
        })
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Flocks>()
        .init_resource::<super::fireflies::Swarms>()
        .add_systems(Update, carry::carry);
    let world = app.world_mut();
    world.spawn((Player, Rebased, Transform::from_xyz(95.0, 3.0, -60.0)));
    world
        .run_system_once(pool::build)
        .expect("the particle looks");
    world
        .run_system_once(gear::build)
        .expect("the creature gear");
    world.run_system_once(pool::spawn).expect("the particles");
    world.run_system_once(super::spawn).expect("the creatures");
    app
}

/// Returns how many entities of the world carry component `T`, and how many of them are free
/// root entities marked to follow a rebase.
fn marked<T: Component>(world: &mut World) -> (usize, usize) {
    let all = world.query_filtered::<(), With<T>>().iter(world).count();
    let free = world.query_filtered::<(), (With<T>, Pooled)>();
    (all, { free }.iter(world).count())
}

/// Every pooled particle and creature is a free root entity marked `Rebased`, in today's numbers.
#[test]
fn every_pooled_body_follows_rebases() {
    let mut app = app();
    let world = app.world_mut();
    assert_eq!(marked::<Particle>(world), (POOL_SIZE, POOL_SIZE));
    assert_eq!(marked::<Firefly>(world), (24, 24));
    assert_eq!(marked::<Butterfly>(world), (12, 12));
    assert_eq!(marked::<Bird>(world), (13, 13));
}

/// Scatters the pooled bodies and their flight targets around the player and returns the
/// planet position of every body, target and flock center under the current anchor.
fn planet_points(world: &mut World, scatter: bool) -> Vec<DVec3> {
    let anchor = *world.resource::<Anchor>();
    let mut points = Vec::new();
    let mut bodies = world.query_filtered::<(&mut Transform, Option<&mut Wander>), Pooled>();
    for (index, (mut at, wander)) in bodies.iter_mut(world).enumerate() {
        let spot = Vec3::new(95.0, 3.0, -60.0) + Vec3::new(index as f32 * 0.7, 9.0, -41.0);
        if scatter {
            at.translation = spot;
        }
        points.push(anchor.to_frame(at.translation));
        if let Some(mut wander) = wander {
            wander.target = if scatter { spot.zyx() } else { wander.target };
            points.push(anchor.to_frame(wander.target));
        }
    }
    let mut flocks = world.resource_mut::<Flocks>();
    for (index, flock) in flocks.0.iter_mut().enumerate() {
        flock.center = match scatter {
            true => Vec3::new(120.0, 30.0 + index as f32, 14.0),
            false => flock.center,
        };
        points.push(anchor.to_frame(flock.center));
    }
    points
}

/// A rebase by the frame systems keeps every creature, every flight target and every flock
/// center within 1 mm of its place on the planet.
#[test]
fn a_rebase_keeps_every_creature_in_place() {
    let mut app = app();
    let world = app.world_mut();
    let old = *world.resource::<Anchor>();
    let before = planet_points(world, true);
    world.run_schedule(Update);
    world.run_schedule(FixedFirst);
    world.run_schedule(Update);
    assert_ne!(*world.resource::<Anchor>(), old);
    let after = planet_points(world, false);
    assert!(before.len() > 200 && before.len() == after.len());
    for (was, now) in before.iter().zip(&after) {
        assert!(was.distance(*now) < 1e-3, "{}", was.distance(*now));
    }
}
