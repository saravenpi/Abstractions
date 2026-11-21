//! Tests of seen marking: plants count as seen on the planet, a creature within 40 m of the
//! robot marks its species, one farther away does not, and the seen species survive a save.

use std::sync::Arc;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::rig::web;
use super::seen::{SIGHT, Seen, sight, sighted, store};
use crate::fauna::Creature;
use crate::life::Guild;
use crate::player::Player;
use crate::saves::{ActiveWorld, WorldEntry, WorldSave};
use crate::universe::{EARTH_RADIUS, STANDARD_GRAVITY};
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
    let globe = Globe::new(1, EARTH_RADIUS, STANDARD_GRAVITY).with_life(Some(life.clone()));
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

/// The seen species go into the world file through toml and come back out as the same set.
#[test]
fn seen_species_survive_a_save_round_trip() {
    let life = web();
    let ids: Vec<_> = life.species.iter().map(|s| s.id).take(3).collect();
    let mut app = App::new();
    app.insert_resource(Seen::default())
        .insert_resource(ActiveWorld(WorldEntry {
            id: "seen".to_string(),
            save: WorldSave::default(),
        }));
    for id in ids.iter().rev() {
        app.world_mut().resource_mut::<Seen>().mark(*id);
    }
    app.world_mut().run_system_once(store).expect("store runs");
    let text = toml::to_string(&app.world().resource::<ActiveWorld>().0.save).unwrap();
    let save: WorldSave = toml::from_str(&text).unwrap();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(save.seen, sorted);
    let mut restored = Seen::default();
    for id in &save.seen {
        restored.mark(*id);
    }
    assert_eq!(&restored, app.world().resource::<Seen>());
}

/// What is seen is per player: with the robot of another player in the world, standing next to a
/// creature the local robot is too far to see, the local robot still marks the creature near itself
/// and does not mark the one the other player sees.
#[test]
fn a_species_is_seen_by_the_local_robot_with_another_player_present() {
    use crate::avatar::{Avatar, AvatarPose};
    let life = web();
    let animals: Vec<_> = life
        .species
        .iter()
        .filter(|s| s.role.guild != Guild::Producer)
        .collect();
    let globe = Globe::new(1, EARTH_RADIUS, STANDARD_GRAVITY).with_life(Some(life.clone()));
    let mut app = App::new();
    app.insert_resource(Terrain::on(Arc::new(globe), Chart::Flat))
        .init_resource::<Seen>();
    let world = app.world_mut();
    world.spawn((Player, GlobalTransform::from_xyz(5.0, 0.0, 5.0)));
    let far = SIGHT * 3.0;
    let other = GlobalTransform::from_xyz(far, 0.0, 5.0);
    world.spawn((Avatar, AvatarPose::default(), other));
    for (slot, x) in [20.0, far + 3.0].into_iter().enumerate() {
        let (species, slot) = (animals[slot].id, slot as u8);
        let creature = Creature {
            species,
            herd: 7,
            slot,
        };
        world.spawn((creature, GlobalTransform::from_xyz(x, 2.0, 10.0)));
    }
    world.run_system_once(sight).expect("sight runs");
    let seen = app.world().resource::<Seen>();
    assert!(seen.has(animals[0].id) && !seen.has(animals[1].id));
}
