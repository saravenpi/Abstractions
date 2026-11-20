//! Tests of the fleck pool: slots, flight and the look of each colour.

use bevy::prelude::*;

use super::fly::step;
use super::pool::{Flecks, claim};

/// A new fleck takes the free slot, else the one that flew longest, and never a slot past the end.
#[test]
fn claims_free_then_oldest() {
    let ages = [Some(1.0), None, Some(3.0)];
    assert_eq!(claim(ages.into_iter()), 1);
    let full = [Some(1.0), Some(5.0), Some(3.0)];
    assert_eq!(claim(full.into_iter()), 1);
    assert!(claim((0..192).map(|i| Some(i as f32))) < 192);
}

/// A fleck thrown level falls under gravity and slows in air, and flies straight in a vacuum.
#[test]
fn falls_and_slows() {
    let (mut at, mut v) = (Vec3::new(0.0, 2.0, 0.0), Vec3::new(5.0, 0.0, 0.0));
    let pull = Vec3::new(0.0, -9.81, 0.0);
    for _ in 0..30 {
        (at, v) = step(at, v, (pull, 1.0), (0.006, 1.0 / 60.0));
    }
    assert!(at.y < 2.0 && v.x < 5.0 && at.x > 0.0);
    let (_, vacuum) = step(Vec3::ZERO, Vec3::X * 5.0, (Vec3::ZERO, 0.0), (0.006, 0.1));
    assert_eq!(vacuum, Vec3::X * 5.0);
}

/// Two flecks of one colour share a material, a second colour makes a second one.
#[test]
fn one_material_per_colour() {
    let mut materials = Assets::<StandardMaterial>::default();
    let mut flecks = Flecks::default();
    let a = flecks.look(&mut materials, [0.4, 0.0, 0.0], true);
    let b = flecks.look(&mut materials, [0.4, 0.0, 0.0], true);
    let c = flecks.look(&mut materials, [0.0, 0.2, 0.6], true);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(materials.len(), 2);
}

/// However many flecks are thrown, the pool keeps its slots and never has more flying.
#[test]
fn the_pool_never_exceeds_its_size() {
    use bevy::ecs::system::RunSystemOnce;

    use super::pool::{Fleck, SLOTS, spawn};
    use super::{Fling, fly::fling};
    let mut app = App::new();
    app.add_message::<Fling>()
        .init_resource::<Flecks>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>();
    let world = app.world_mut();
    world.run_system_once(spawn).expect("the flecks");
    for n in 0..SLOTS * 3 {
        world.write_message(Fling {
            at: Vec3::splat(n as f32),
            velocity: Vec3::Y,
            radius: 0.006,
            colour: [0.4, 0.0, 0.0],
            wet: true,
            tag: Some(1),
        });
    }
    world.run_system_once(fling).expect("the throw");
    let all = world.query::<&Fleck>().iter(world).count();
    let live = world
        .query::<&Fleck>()
        .iter(world)
        .filter(|f| f.live)
        .count();
    assert_eq!(all, SLOTS);
    assert_eq!(live, SLOTS);
    assert!(world.resource::<Flecks>().live <= SLOTS);
}
