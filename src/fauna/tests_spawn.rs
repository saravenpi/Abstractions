//! Tests of which animals are drawn: they follow the herds of the planet's life, near the robot only,
//! never more than the cap, none on a dead planet, and a rebase leaves them in place.

use bevy::prelude::*;

use super::beast::Beast;
use super::gate::living;
use super::herds::herd_arc;
use super::rig::app;
use super::stage::{put, robot, run, staying};
use super::tune::{MOST, REACH, SLACK};
use crate::space::{Anchor, Placed};

/// A living planet draws the members of the herd the robot stands in, at most [`MOST`], all within
/// reach of the robot.
#[test]
fn animals_follow_the_herds() {
    let (mut app, herd) = app(true);
    let herd = herd.expect("a herd on the living home planet");
    run(&mut app, 1.0, 0.1);
    let beasts = staying(&mut app);
    let near = robot(&mut app).normalize();
    let radius = app
        .world()
        .resource::<crate::world::Terrain>()
        .globe()
        .radius();
    println!(
        "{} animals drawn, herd {} of {}",
        beasts.len(),
        herd.id,
        herd.count
    );
    assert!(!beasts.is_empty() && beasts.len() <= MOST);
    assert!(beasts.iter().any(|b| b.lead.key().0 == herd.id));
    for b in &beasts {
        let metres = herd_arc(b.spot.planet(b.motion.root.at).normalize(), near) * radius;
        assert!(metres <= REACH + SLACK + 5.0, "{metres} m from the robot");
    }
}

/// A dead planet draws nothing, and the systems do not run there.
#[test]
fn a_dead_planet_has_no_animals() {
    let (mut app, _) = app(false);
    run(&mut app, 1.0, 0.1);
    assert!(staying(&mut app).is_empty());
    let alive = app
        .world_mut()
        .run_system_cached(living)
        .expect("the condition runs");
    assert!(!alive);
}

/// Animals fade out and are gone once the robot walks far away.
#[test]
fn far_animals_fade_out() {
    let (mut app, _) = app(true);
    run(&mut app, 1.0, 0.1);
    let before: Vec<Entity> = drawn(&mut app);
    assert!(!before.is_empty());
    let away = robot(&mut app) + bevy::math::DVec3::new(900.0, 0.0, 0.0);
    put(&mut app, away);
    run(&mut app, 2.0, 0.1);
    let after = drawn(&mut app);
    assert!(
        before.iter().all(|e| !after.contains(e)),
        "old animals stay"
    );
}

/// Returns every drawn animal entity.
fn drawn(app: &mut App) -> Vec<Entity> {
    let world = app.world_mut();
    let mut beasts = world.query_filtered::<Entity, With<Beast>>();
    beasts.iter(world).collect()
}

/// A new anchor leaves every animal where it was on the planet, within a millimetre.
#[test]
fn a_rebase_keeps_animals_in_place() {
    let (mut app, _) = app(true);
    run(&mut app, 1.0, 0.1);
    let places = |app: &mut App| {
        let world = app.world_mut();
        let anchor = *world.resource::<Anchor>();
        let mut roots = world.query_filtered::<&Transform, With<Placed>>();
        let roots: Vec<_> = roots
            .iter(world)
            .map(|t| anchor.to_frame(t.translation))
            .collect();
        roots
    };
    let before = places(&mut app);
    let world = app.world_mut();
    let mut anchor = *world.resource::<Anchor>();
    anchor.origin += anchor.rotation.inverse() * bevy::math::DVec3::new(60.0, 0.0, -35.0);
    world.insert_resource(anchor);
    world.run_schedule(PostUpdate);
    let after = places(&mut app);
    assert!(!before.is_empty() && before.len() == after.len());
    for (was, now) in before.iter().zip(&after) {
        assert!(was.distance(*now) < 1e-3, "{} m", was.distance(*now));
    }
}
