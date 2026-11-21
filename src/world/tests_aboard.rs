//! Tests of the start of a world saved in the seat of the ship: the ship says where the player
//! is, so the body is kept, ground or not, and the place is kept in orbit.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;

use super::enter::{Resettled, arrive};
use crate::saves::{ActiveWorld, Place, WorldEntry, WorldSave};
use crate::universe::{BodyId, PlanetClass, PlanetSpec, Universe};

/// Returns a world of seed 7 saved at `place`, with a ship record that only says whether the
/// player is `seated`.
fn saved_at(place: Place, seated: bool) -> ActiveWorld {
    let ship = [("seated".to_string(), toml::Value::Boolean(seated))];
    let save = WorldSave {
        seed: 7,
        place: Some(place),
        ship: Some(toml::Value::Table(ship.into_iter().collect())),
        ..WorldSave::default()
    };
    let id = "w".to_string();
    ActiveWorld(WorldEntry { id, save })
}

/// Starts the world of `active` and returns the body it starts on and the line left for the player.
fn begun(active: &mut ActiveWorld) -> (BodyId, Option<Resettled>) {
    let universe = Universe::new(7);
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);
    let (body, _) = arrive(&mut commands, &universe, Some(&mut &mut *active));
    queue.apply(&mut world);
    (body, world.get_resource::<Resettled>().copied())
}

/// A save seated in the ship keeps its body and its ship even over a gas giant, and its place
/// is kept in orbit, without a word. On foot the same place starts at the base.
#[test]
fn a_seated_save_keeps_a_body_without_ground() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let gas = |planet: &&PlanetSpec| planet.record.class == PlanetClass::GasGiant;
    let giant = system.planets.iter().find(gas).expect("a gas giant");
    let place = Place {
        body: giant.id,
        dir: [0.6, 0.0, -0.8],
        altitude: giant.radius * 0.3,
    };
    let mut aboard = saved_at(place, true);
    assert_eq!(begun(&mut aboard), (giant.id, None));
    assert_eq!(aboard.0.save.place, Some(place));
    assert!(aboard.0.save.ship.is_some());
    let mut afoot = saved_at(place, false);
    assert_eq!(begun(&mut afoot).0, universe.home());
    assert_eq!((afoot.0.save.place, &afoot.0.save.ship), (None, &None));
}

/// A save seated in open space, far past where the planet is drawn, loads just past that
/// distance over the same body, and is not put on the ground.
#[test]
fn a_seated_save_far_away_loads_clear_of_the_planet() {
    let home = Universe::new(7).home();
    let far = Place {
        body: home,
        dir: [0.0, 1.0, 0.0],
        altitude: 4.0e11,
    };
    let mut active = saved_at(far, true);
    assert_eq!(begun(&mut active), (home, None));
    let place = active.0.save.place.expect("a place");
    let radius = Universe::new(7).planet(home).radius;
    assert!(
        place.altitude > radius * 0.4 && place.altitude < radius,
        "{}",
        place.altitude
    );
}
