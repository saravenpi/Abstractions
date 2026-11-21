//! Tests of the place a world starts at: a saved place no body can be at is put on the ground or
//! dropped before the world begins, and a place in the air is kept.

use bevy::ecs::world::CommandQueue;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Globe;
use super::enter::{AT_BASE, GROUNDED, Resettled, arrive};
use crate::saves::{ActiveWorld, Place, WorldEntry, WorldSave};
use crate::universe::{BodyId, Universe};

/// Returns a world of seed 4 saved at `place`, with a ship record.
fn saved_at(place: Place) -> ActiveWorld {
    let save = WorldSave {
        seed: 4,
        place: Some(place),
        ship: Some(toml::Value::String("ship".to_string())),
        ..WorldSave::default()
    };
    let id = "w".to_string();
    ActiveWorld(WorldEntry { id, save })
}

/// Starts the world of `active` and returns the body it starts on and the line left for the player.
fn begun(active: &mut ActiveWorld) -> (BodyId, Option<Resettled>) {
    let universe = Universe::new(4);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let (body, _) = arrive(&mut commands, &universe, Some(&mut &mut *active));
    queue.apply(&mut world);
    (body, world.get_resource::<Resettled>().copied())
}

/// Returns the height in metres of the ground or the sea of the home planet of seed 4 in the direction
/// the tests stand in.
fn floor() -> f64 {
    let universe = Universe::new(4);
    let globe = Globe::of(&universe.planet(universe.home()));
    f64::from(globe.height(dir()).max(globe.sea_level()))
}

/// Returns the direction the tests stand in.
fn dir() -> DVec3 {
    DVec3::new(0.2, 1.0, 0.1).normalize()
}

/// Returns a place `altitude` metres above the sea level of the home planet of seed 4.
fn home_at(altitude: f64) -> Place {
    Place {
        body: Universe::new(4).home(),
        dir: dir().to_array(),
        altitude,
    }
}

/// A place in the air on a body that exists is kept as it is, and nothing is told.
#[test]
fn a_place_in_the_air_is_kept() {
    let mut active = saved_at(home_at(floor() + 6560.18));
    let before = active.clone();
    assert_eq!(
        begun(&mut active),
        (before.0.save.body(&Universe::new(4)), None)
    );
    assert_eq!(active, before);
}

/// A place far under the ground, as a server repairs a bad height to, starts on the ground in
/// the same direction, the ship kept, and the player is told.
#[test]
fn a_place_under_the_ground_starts_on_the_ground() {
    let low = home_at(-20_000.0);
    let mut active = saved_at(low);
    let (body, line) = begun(&mut active);
    assert_eq!((body, line), (low.body, Some(Resettled(GROUNDED))));
    let place = active.0.save.place.expect("a place");
    assert_eq!((place.body, place.dir), (low.body, low.dir));
    assert!(
        (place.altitude - floor() - 1.5).abs() < 1.0e-6,
        "{}",
        place.altitude
    );
    assert!(active.0.save.ship.is_some());
}

/// A place on a body that does not exist, or one that points nowhere, is dropped with its ship:
/// the world begins at the home planet as a new game, and the player is told.
#[test]
fn a_place_on_no_body_starts_at_the_base() {
    let home = Universe::new(4).home();
    let mut nowhere = home_at(3.0);
    nowhere.body.star.index = 250;
    let pointless = Place {
        dir: [f64::NAN, 0.0, 0.0],
        ..home_at(3.0)
    };
    for place in [nowhere, pointless] {
        let mut active = saved_at(place);
        assert_eq!(begun(&mut active), (home, Some(Resettled(AT_BASE))));
        let save = &active.0.save;
        assert_eq!((save.place, &save.ship), (None, &None));
    }
}
