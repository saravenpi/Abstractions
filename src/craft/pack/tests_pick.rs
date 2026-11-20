//! Tests of picking stones up: a rock under the crosshair gives a piece of the stone its id picks, at
//! most a handful and at most the room, and keeps the rest. Nothing under the crosshair gives nothing.

use bevy::prelude::*;

use super::Pack;
use super::pick::{stone_of, stones, take};
use super::tune::HANDFUL;
use crate::craft::matter::Stuff;
use crate::universe::{PlanetRecord, Universe};
use crate::world::rocks::{Plant, Rock, RockId, Taken};

/// Returns the record of the home planet of a world.
fn home() -> PlanetRecord {
    let universe = Universe::new(1);
    universe.planet(universe.home()).record
}

/// Returns a rock with the id `id` drawn at `scale`.
fn rock(id: u64, scale: f32) -> Rock {
    Rock {
        id: RockId(id),
        plant: Plant::Rock,
        scale,
        at: Vec3::ZERO,
        mid: Vec3::Y * 0.45,
        left: 1.0,
    }
}

/// With no rock under the ray, nothing is stowed and nothing is taken.
#[test]
fn no_rock_under_the_ray_stows_nothing() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let hit = take(None, (&home(), 25.0), (&mut pack, &mut taken));
    assert_eq!(hit, None);
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
        let stone = stone_of(&record, RockId(n));
        assert_eq!(stone, Some(found[n as usize % found.len()]));
        let (mut pack, mut taken) = (Pack::default(), Taken::default());
        take(Some(rock(n, 1.0)), (&record, 25.0), (&mut pack, &mut taken));
        assert_eq!(pack.stacks[0].stuff, Stuff::Mineral(stone.unwrap()));
    }
}

/// A pick takes at most a handful, and the rock keeps the rest: its taken mass grows by the piece.
#[test]
fn a_pick_takes_at_most_a_handful_and_the_rock_keeps_the_rest() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let big = rock(4, 1.0);
    let hit = take(Some(big), (&home(), 25.0), (&mut pack, &mut taken));
    assert_eq!(hit, Some((RockId(4), HANDFUL)));
    assert!((pack.mass() - HANDFUL).abs() < 1.0e-4);
    assert!((taken.kilograms(RockId(4)) - HANDFUL).abs() < 1.0e-4);
    assert!(taken.left(RockId(4)) < 1.0 && taken.left(RockId(4)) > 0.99);
    take(Some(big), (&home(), 25.0), (&mut pack, &mut taken));
    assert!((taken.kilograms(RockId(4)) - 2.0 * HANDFUL).abs() < 1.0e-4);
}

/// A pick takes no more than the room left in the pack, and nothing when there is none.
#[test]
fn a_pick_takes_at_most_the_room_and_nothing_without_it() {
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    take(Some(rock(4, 1.0)), (&home(), 1.0), (&mut pack, &mut taken));
    assert!((pack.mass() - 1.0).abs() < 1.0e-4);
    let (mut full, mut kept) = (Pack::default(), Taken::default());
    let hit = take(Some(rock(4, 1.0)), (&home(), 0.0), (&mut full, &mut kept));
    assert_eq!(hit, None);
    assert!(full.stacks.is_empty());
    assert_eq!(kept, Taken::default());
}

/// A rock small enough to carry goes whole, and is gone: it weighs its volume times its density.
#[test]
fn a_small_rock_goes_whole_and_is_gone() {
    let record = home();
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let small = rock(4, 0.1);
    let hit = take(Some(small), (&record, 25.0), (&mut pack, &mut taken));
    let (id, kilograms) = hit.expect("a piece");
    let density = pack.stacks[0].material.density;
    let volume = 2.0 / 3.0 * std::f32::consts::PI * 0.07 * 0.07 * 0.09;
    assert!((kilograms - volume * density).abs() < 1.0e-3);
    assert!(kilograms < HANDFUL);
    assert_eq!(taken.left(id), 0.0);
}

/// A rock is gone only when its mass is used up: a pick at a time, the handful of each adds up to the
/// whole rock and not a gram more.
#[test]
fn picks_add_up_to_the_whole_rock() {
    let record = home();
    let (mut pack, mut taken) = (Pack::default(), Taken::default());
    let small = rock(4, 0.3);
    let mut picks = 0;
    while taken.left(RockId(4)) > 0.0 && picks < 1000 {
        take(Some(small), (&record, 100.0), (&mut pack, &mut taken));
        picks += 1;
    }
    let whole = taken.whole(RockId(4)).expect("a quarry");
    assert!(picks > 1);
    assert!((pack.mass() - whole).abs() < 1.0e-2);
}
