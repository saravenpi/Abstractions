//! The world a welcome describes and the record sent back: the atlas, the place and the ship
//! survive the trip as TOML text, and a damaged atlas falls back to a new game's.

use abstractions_net::log::Changes;

use super::join::{moon, visited};
use super::rig::welcome;
use crate::computer::Atlas;
use crate::net::join::world::{atlas_of, record_of, world_of};
use crate::saves::{Place, WorldSave};
use crate::universe::Universe;

/// An empty or damaged atlas is the atlas of a new game; a written one reads back.
#[test]
fn the_atlas_reads_back_or_starts_new() {
    let universe = Universe::new(8);
    assert_eq!(atlas_of("", &universe), Atlas::new(&universe));
    assert_eq!(atlas_of("bodies = 3", &universe), Atlas::new(&universe));
    let mut atlas = Atlas::new(&universe);
    atlas.walked.clear();
    assert_eq!(
        atlas_of(&toml::to_string(&atlas).unwrap(), &universe),
        atlas
    );
}

/// The place and ship a save captured go to the server as text and come back in a welcome.
#[test]
fn the_record_survives_the_trip() {
    let mut ship = toml::Table::new();
    ship.insert("seated".to_string(), toml::Value::Boolean(true));
    ship.insert("speed".to_string(), toml::Value::Float(2.5));
    let save = WorldSave {
        place: Some(Place {
            body: moon(8, 1),
            dir: [1.0, 0.0, 0.0],
            altitude: 12.0,
        }),
        ship: Some(toml::Value::Table(ship)),
        ..WorldSave::default()
    };
    let record = record_of(&save);
    let back = world_of(&welcome(8, Changes::default(), Some(record)), "Space");
    assert_eq!((back.save.place, back.save.ship), (save.place, save.ship));
    assert_eq!(back.save.name, "Space");
}

/// A first visit has no record: the player starts at the normal spawn of a new world, and
/// the atlas holds what the log says was visited.
#[test]
fn a_first_visit_starts_new() {
    let back = world_of(&welcome(8, visited(moon(8, 2)), None), "Space");
    assert_eq!((back.save.place, back.save.ship), (None, None));
    assert_eq!(back.save.play_seconds, 0.0);
    let bodies = back.save.atlas.unwrap().bodies;
    assert_eq!(bodies, [Universe::new(8).home(), moon(8, 2)]);
}
