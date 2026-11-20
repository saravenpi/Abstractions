//! Tests of seen marking: plants count as seen on the planet, a creature within 40 m of the
//! robot marks its species, and one farther away does not.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::rig::web;
use super::seen::{SIGHT, Seen, sight, sighted};
use crate::fauna::Creature;
use crate::life::Guild;
use crate::player::Player;
use crate::universe::{HOME_GRAVITY, HOME_RADIUS};
use crate::world::globe::Chart;
use crate::world::{Globe, Terrain};

/// Plants are always in sight; a creature is in sight within 40 m only.
#[test]
fn sight_reaches_forty_metres() {
    let life = web();
    let animals: Vec<_> = life
        .species
        .iter()
        .filter(|s| s.role.guild != Guild::Producer)
        .collect();
    let plants = life.species.len() - animals.len();
    let near = (animals[0].id, Vec3::new(SIGHT - 1.0, 0.0, 0.0));
    let far = (animals[1].id, Vec3::new(0.0, 0.0, SIGHT + 1.0));
    let found: Vec<_> = sighted(&life, Vec3::ZERO, [near, far].into_iter(), SIGHT).collect();
    assert_eq!(found.len(), plants + 1);
    assert!(found.contains(&animals[0].id) && !found.contains(&animals[1].id));
}

/// A hurt head shortens the reach of the scan.
#[test]
fn a_hurt_head_sees_less_far() {
    let life = web();
    let animal = life
        .species
        .iter()
        .find(|s| s.role.guild != Guild::Producer)
        .expect("an animal");
    let near = (animal.id, Vec3::new(SIGHT - 5.0, 0.0, 0.0));
    let found = |reach: f32| sighted(&life, Vec3::ZERO, [near].into_iter(), reach).count();
    let plants = found(0.0);
    assert_eq!(found(SIGHT), plants + 1);
    assert_eq!(found(SIGHT * crate::condition::tune::HEAD_SCAN), plants);
}

/// In the running game a creature near the robot on a living planet is marked seen.
#[test]
fn a_near_creature_is_marked_seen() {
    let life = web();
    let animals: Vec<_> = life
        .species
        .iter()
        .filter(|s| s.role.guild != Guild::Producer)
        .collect();
    let globe = Globe::new(1, HOME_RADIUS, HOME_GRAVITY).with_life(Some(life.clone()));
    let mut app = App::new();
    app.insert_resource(Terrain::on(Arc::new(globe), Chart::Flat))
        .init_resource::<Seen>();
    let world = app.world_mut();
    world.spawn((Player, GlobalTransform::from_xyz(5.0, 0.0, 5.0)));
    for (slot, (animal, x)) in [(animals[0], 20.0), (animals[1], 90.0)]
        .into_iter()
        .enumerate()
    {
        let creature = Creature {
            species: animal.id,
            herd: 7,
            slot: slot as u8,
        };
        world.spawn((creature, GlobalTransform::from_xyz(x, 2.0, 10.0)));
    }
    world.run_system_once(sight).expect("sight runs");
    let seen = app.world().resource::<Seen>();
    assert!(seen.has(animals[0].id) && !seen.has(animals[1].id));
    assert_eq!(seen.count(&life), life.species.len() - animals.len() + 1);
}
