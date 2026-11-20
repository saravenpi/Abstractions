//! Tests of the walk: animals stand and step on the drawn ground, and its cost per frame.

use std::hint::black_box;

use bevy::prelude::*;

use super::beast::Beast;
use super::rig::app;
use super::stage::{robot, run, staying};
use super::tune::{MOST, NEAR};
use super::walk::walk;
use super::want::Lead;
use crate::life::body::Locomotion;
use crate::world::Terrain;

/// Returns the locomotion of the species of `beast` on the planet of `terrain`.
fn moves(terrain: &Terrain, beast: &Beast) -> Option<Locomotion> {
    let life = terrain.globe().life()?;
    Some(life.species.get(usize::from(beast.index))?.body.locomotion)
}

/// Near walkers keep their planted feet on the drawn ground within 3 cm as they move about, and every
/// walker its drawn origin within 25 cm, the bob of its gait and the lag of its body over its feet.
#[test]
fn animals_stand_on_the_ground() {
    let (mut app, _) = app(true);
    run(&mut app, 3.0, 1.0 / 30.0);
    let robot = robot(&mut app);
    let beasts = staying(&mut app);
    let terrain = app.world().resource::<Terrain>().clone();
    let walkers = beasts
        .iter()
        .filter(|b| !matches!(b.lead, Lead::Carcass { .. }));
    let walkers: Vec<&Beast> = walkers
        .filter(|b| moves(&terrain, b) == Some(Locomotion::Walk))
        .collect();
    assert!(!walkers.is_empty());
    let (mut worst, mut feet) = (0.0_f32, 0);
    for b in &walkers {
        let (at, shown) = (b.motion.root.at, b.motion.root.place);
        worst = worst.max((shown.y - b.spot.ground(&terrain, shown.xz())).abs());
        if b.spot.planet(at).distance(robot) < f64::from(NEAR) {
            for f in b.motion.feet.iter().filter(|f| f.planted) {
                let gap = (f.at.y - b.spot.ground(&terrain, f.at.xz())).abs();
                assert!(gap < 0.03, "a planted foot {gap} m off the ground");
                feet += 1;
            }
        }
    }
    println!(
        "{} walkers, worst root {worst:.3} m off the ground, {feet} planted feet",
        walkers.len()
    );
    assert!(worst < 0.25, "a walker {worst} m off the ground");
}

/// One frame of the walk over the drawn animals costs well under a millisecond for [`MOST`] of them at
/// opt-level 1, scaled from the animals of the rig.
#[test]
fn the_walk_is_cheap() {
    let (mut app, _) = app(true);
    run(&mut app, 2.0, 0.1);
    let count = staying(&mut app).len();
    let mut schedule = Schedule::default();
    schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    schedule.add_systems(walk);
    let world = app.world_mut();
    schedule.run(world);
    let rounds = 120;
    let start = super::tests_react::busy();
    for _ in 0..rounds {
        world
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        schedule.run(black_box(&mut *world));
    }
    let ms = (super::tests_react::busy() - start) * 1.0e3 / f64::from(rounds);
    let full = ms * MOST as f64 / count.max(1) as f64;
    println!("walk: {ms:.3} ms per frame for {count} animals, {full:.3} ms for {MOST}");
    assert!(
        count > 0 && full < 1.0,
        "{full} ms per frame for {MOST} animals"
    );
}

/// The ground cache reads the same drawn ground as the terrain, within a millimetre, at any chart
/// position and again once cached.
#[test]
fn cached_ground_is_the_drawn_ground() {
    let (app, _) = app(true);
    let terrain = app.world().resource::<Terrain>();
    let heights = super::ground::Heights::default();
    for round in 0..2 {
        for k in 0..400 {
            let (x, z) = (k as f32 * 0.37 - 70.0, (k * 7 % 400) as f32 * 0.29 - 55.0);
            let gap = (heights.mesh(terrain, x, z) - terrain.mesh_height(x, z)).abs();
            assert!(gap < 1e-3, "round {round}: {gap} m at {x}, {z}");
        }
    }
}
