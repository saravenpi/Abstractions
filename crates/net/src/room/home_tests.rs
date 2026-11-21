//! Tests of the home rule a room hands its players: the one its world was made under.

use super::map_tests::welcome_of_cyd;
use super::{Room, RoomConfig};
use crate::store::{HOME_RULE, World};

/// Returns a room of a world of seed 77 made under the home `rule`, at network time 100.
fn room_of(rule: u32) -> Room {
    let config = RoomConfig {
        name: "Test".to_string(),
        password: None,
        cap: 16,
        version: "0.9.0".to_string(),
    };
    let world = World {
        seed: 77,
        home: rule,
        ..World::default()
    };
    Room::new(config, world, 100.0)
}

/// A player who joins gets the seed with the home rule of the world, the first rule of an old
/// world as the Earth-like rule of a new one, and the world saved after keeps it.
#[test]
fn the_welcome_carries_the_home_rule() {
    for rule in [0, HOME_RULE] {
        let mut room = room_of(rule);
        let welcome = welcome_of_cyd(&mut room, 100.0);
        assert_eq!((welcome.seed, welcome.home), (77, rule));
        assert_eq!(room.snapshot(200.0).home, rule);
    }
}
