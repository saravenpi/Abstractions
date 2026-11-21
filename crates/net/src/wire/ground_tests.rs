//! Tests of the ground on the wire: every act and event about it comes back, the numbers of the
//! old acts and events have not moved, and a thing reads back from the text of a world file.

use super::hunt_tests::size;
use super::{Act, ClientMsg, Event, HEAP, LYING_BYTES, Lying, Rock, ServerMsg, SpearKey, TOOL};
use crate::ground::tests::thing;
use crate::ids::BodyId;
use crate::log::PlayerId;

/// Returns the biggest thing a server takes: all of its bytes used.
fn biggest() -> Lying {
    Lying {
        material: vec![3; 40],
        item: vec![7; LYING_BYTES - 40],
        ..thing(TOOL, 1, 1.5, 12.0)
    }
}

/// Every act and event of the ground comes back, and the biggest drop stays within its bytes and
/// a small head.
#[test]
fn every_ground_message_comes_back() {
    assert!(size(&ClientMsg::Hunt(Act::Drop(biggest()))) <= LYING_BYTES + 96);
    size(&ClientMsg::Hunt(Act::Take { id: 7, mass: 2.5 }));
    size(&ServerMsg::Hunt(Event::Lies(biggest())));
    let gone = Event::Gone {
        id: 7,
        by: PlayerId(2),
    };
    size(&ServerMsg::Hunt(gone));
    size(&ServerMsg::Hunt(Event::Granted(thing(HEAP, 2, 0.5, 1.0))));
    size(&ServerMsg::Hunt(Event::Denied(7)));
    size(&ServerMsg::Hunt(Event::Kept(3)));
    size(&ServerMsg::Hunt(Event::Ground(vec![biggest(); 3])));
}

/// The acts and events of the ground come after the old ones, whose numbers stay.
#[test]
fn old_numbers_stay() {
    let first = |bytes: Vec<u8>| bytes[0];
    let rock = Rock {
        planet: BodyId::default(),
        id: 1,
        grams: 2,
        whole: 3,
    };
    let key = SpearKey {
        owner: PlayerId(1),
        tag: 2,
    };
    let act = |act: &Act| first(postcard::to_stdvec(act).unwrap());
    let event = |event: &Event| first(postcard::to_stdvec(event).unwrap());
    assert_eq!(act(&Act::Pull(key)), 5);
    assert_eq!(act(&Act::Rock(rock)), 12);
    assert_eq!(act(&Act::Drop(thing(TOOL, 1, 1.0, 0.0))), 13);
    assert_eq!(act(&Act::Take { id: 1, mass: 1.0 }), 14);
    assert_eq!(event(&Event::Rock(rock)), 13);
    assert_eq!(event(&Event::Lies(thing(TOOL, 1, 1.0, 0.0))), 14);
    assert_eq!(event(&Event::Ground(Vec::new())), 19);
}

/// A thing written in the text of a world file shows its bytes in hexadecimal and reads back
/// unchanged; text that is not hexadecimal is refused.
#[test]
fn a_thing_reads_back_from_text() {
    let lying = Lying {
        material: vec![0x0a, 0xff],
        ..thing(HEAP, 1, 2.0, 3.0)
    };
    let text = toml::to_string(&lying).unwrap();
    assert!(text.contains("material = \"0aff\""), "{text}");
    assert_eq!(toml::from_str::<Lying>(&text).unwrap(), lying);
    let broken = text.replace("\"0aff\"", "\"0az\"");
    assert!(toml::from_str::<Lying>(&broken).is_err());
}
