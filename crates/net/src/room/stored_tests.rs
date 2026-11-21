//! Tests of the check of the place the room keeps of a player: repaired when its player joins
//! and when a game sends it.

use super::kit::{hello, join, messages, place_text, room};
use super::{Room, RoomConfig};
use crate::host::{Host, Outbox};
use crate::store::place::LOWEST;
use crate::store::{Member, World};
use crate::wire::{ClientMsg, PlayerRecord, ServerMsg};

/// Returns a room whose world already keeps `place` for the player of key 1.
fn room_keeping(place: &str) -> Room {
    let member = Member {
        name: "Ada".to_string(),
        place: place.to_string(),
        ..Member::default()
    };
    let world = World {
        players: [("01".repeat(32), member)].into(),
        ..World::default()
    };
    let config = RoomConfig {
        name: "Test".to_string(),
        password: None,
        cap: 4,
        version: "0.11.0".to_string(),
    };
    Room::new(config, world, 100.0)
}

/// Joins the player of key 1 in a room that keeps `stored` for them, and returns the place the
/// welcome hands back and whether the room has something new to save.
fn place_of(stored: &str) -> (String, bool) {
    let mut room = room_keeping(stored);
    assert!(!room.changed());
    let mut out = Outbox::default();
    let conn = join(&mut room, 1, hello("Ada", 1), 110.0, &mut out);
    let record = messages(&out, conn).into_iter().find_map(|msg| match msg {
        ServerMsg::Welcome(welcome) => welcome.record,
        _ => None,
    });
    (record.expect("a record").place, room.changed())
}

/// A stored place with a height no body can be at is handed back on the ground in the same
/// direction when its player joins, one that does not read is forgotten, and a good one stays.
#[test]
fn an_invalid_stored_place_is_repaired_on_join() {
    let grounded = place_text(&format!("{LOWEST:?}"));
    let read = |text: &str| toml::from_str::<toml::Table>(text).expect("a place");
    let (high, changed) = place_of(&place_text("1.0e9"));
    assert_eq!((read(&high), changed), (read(&grounded), true));
    let (nan, _) = place_of(&place_text("nan"));
    assert_eq!(read(&nan), read(&grounded));
    assert_eq!(place_of("p"), (String::new(), true));
    let air = place_text("6560.18");
    let kept = place_of(&air);
    assert_eq!(kept, (air, false));
}

/// A record that comes in with a damaged place is repaired before it is kept.
#[test]
fn an_invalid_place_sent_by_a_game_is_repaired() {
    let mut room = room(4, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let record = PlayerRecord {
        place: place_text("inf"),
        ..PlayerRecord::default()
    };
    room.message(ada, ClientMsg::Record(record), 101.0, &mut out);
    let world = room.snapshot(0.0);
    let kept = &world.players[&"01".repeat(32)].place;
    let table = toml::from_str::<toml::Table>(kept).expect("a place");
    assert_eq!(table["altitude"].as_float(), Some(LOWEST));
    assert_eq!(table["body"]["star"]["sector"].to_string(), "[35, 0, -44]");
}
