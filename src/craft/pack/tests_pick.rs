//! Tests of picking stones up: a rock under the crosshair gives the stone its id picks, as much of it
//! as the robot can carry, and is noted as taken. Nothing under the crosshair gives nothing.

use bevy::prelude::*;

use super::Pack;
use super::pick::{broken, stones, take};
use crate::craft::matter::Stuff;
use crate::universe::{PlanetRecord, Universe};
use crate::world::rocks::{Plant, Rock, RockId, Taken};

/// Returns the record of the home planet of a world.
fn home() -> PlanetRecord {
    let universe = Universe::new(1);
    universe.planet(universe.home()).record
}

/// Returns a rock of scale one with the id `id`.
fn rock(id: u64) -> Rock {
    Rock {
        id: RockId(id),
        plant: Plant::Rock,
        scale: 1.0,
        at: Vec3::ZERO,
        mid: Vec3::Y * 0.45,
    }
}

/// With no rock under the ray, nothing is stowed and nothing is taken.
#[test]
fn no_rock_under_the_ray_stows_nothing() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let id = take(None, (&home(), 25.0), (&mut pack, &mut taken));
    assert_eq!(id, None);
    assert!(pack.stacks.is_empty());
    assert_eq!(taken, Taken::default());
}

/// A rock gives the stone of the planet its id picks, the same one every time.
#[test]
fn a_rock_gives_the_stone_its_id_picks() {
    let record = home();
    let found = stones(&record);
    assert!(!found.is_empty());
    for n in 0..found.len() as u64 * 2 {
        let stack = broken(&record, &rock(n)).expect("a stone");
        assert_eq!(stack.stuff, Stuff::Mineral(found[n as usize % found.len()]));
        assert_eq!(Some(stack), broken(&record, &rock(n)));
    }
}

/// Picking a rock stows its stone up to the room left, notes its id and returns it.
#[test]
fn picking_a_rock_stows_its_stone_and_records_it() {
    let record = home();
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let id = take(Some(rock(4)), (&record, 3.0), (&mut pack, &mut taken));
    assert_eq!(id, Some(RockId(4)));
    assert!(taken.has(RockId(4)));
    let stone = broken(&record, &rock(4)).expect("a stone");
    assert_eq!(pack.stacks.len(), 1);
    assert_eq!(pack.stacks[0].stuff, stone.stuff);
    assert!((pack.mass() - 3.0).abs() < 1.0e-4);
}

/// A robot with no room left leaves the rock where it is.
#[test]
fn a_full_robot_leaves_the_rock() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let id = take(Some(rock(4)), (&home(), 0.0), (&mut pack, &mut taken));
    assert_eq!(id, None);
    assert!(pack.stacks.is_empty());
    assert_eq!(taken, Taken::default());
}

/// A small rock goes whole, and weighs its volume times the density of its stone.
#[test]
fn a_small_rock_goes_whole() {
    let record = home();
    let mut small = rock(4);
    small.scale = 0.1;
    let stack = broken(&record, &small).expect("a stone");
    let volume = 2.0 / 3.0 * std::f32::consts::PI * 0.07 * 0.07 * 0.09;
    assert!((stack.mass - volume * stack.material.density).abs() < 1.0e-3);
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    take(Some(small), (&record, 25.0), (&mut pack, &mut taken));
    assert!((pack.mass() - stack.mass).abs() < 1.0e-4);
}
