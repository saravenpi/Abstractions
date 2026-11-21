//! Tests of the ground the room keeps: a drop gets its id and reaches everyone, the first take wins,
//! a full cube hands the thing back, and a player who joins later, or after a restart, finds it.

use super::hunt_kit::{act, events};
use super::kit::{hello, join};
use super::map_tests::pair;
use super::{Room, RoomConfig};
use crate::ground::CELL_MAX;
use crate::ground::tests::thing;
use crate::host::Outbox;
use crate::log::PlayerId;
use crate::wire::{Act, Event, HEAP, Lying, TOOL};

/// Returns the things that lie, in the order the events of `told` say so.
fn lies(told: &[Event]) -> Vec<Lying> {
    let each = told.iter().filter_map(|event| match event {
        Event::Lies(lying) => Some(vec![lying.clone()]),
        Event::Ground(all) => Some(all.clone()),
        _ => None,
    });
    each.flatten().collect()
}

/// A drop that claims to be Bob's lies with the id the room gives it and Ada's name, and both
/// players hear it, the one who dropped it too.
#[test]
fn a_drop_gets_an_id_and_reaches_everyone() {
    let (mut room, ada, bob, mut out) = pair();
    let claimed = Lying {
        id: 77,
        by: PlayerId(2),
        ..thing(TOOL, 1, 1.2, 0.0)
    };
    act(&mut room, ada, Act::Drop(claimed), (101.0, &mut out));
    let seen = lies(&events(&out, bob));
    assert_eq!(seen, lies(&events(&out, ada)));
    let [lying] = seen.as_slice() else {
        panic!("one thing");
    };
    assert_eq!((lying.id, lying.by, lying.tag), (1, PlayerId(1), 1));
    assert_eq!((lying.since, lying.mass), (5_001.0, 1.2));
}

/// Two players ask for the same thing in the same tick: the first gets it, the second hears it is
/// gone, and both hear it left the ground once.
#[test]
fn two_players_ask_for_one_thing_and_one_gets_it() {
    let (mut room, ada, bob, mut out) = pair();
    act(
        &mut room,
        ada,
        Act::Drop(thing(TOOL, 1, 1.0, 0.0)),
        (101.0, &mut out),
    );
    out.clear();
    let take = Act::Take { id: 1, mass: 25.0 };
    act(&mut room, bob, take.clone(), (102.0, &mut out));
    act(&mut room, ada, take, (102.0, &mut out));
    let gone = Event::Gone {
        id: 1,
        by: PlayerId(2),
    };
    let bobs = events(&out, bob);
    assert!(matches!(bobs.as_slice(), [Event::Granted(got), g] if got.id == 1 && *g == gone));
    assert_eq!(events(&out, ada), [gone, Event::Denied(1)]);
    assert!(room.snapshot(103.0).ground.is_empty());
}

/// A take of part of a heap gives that part and tells everyone what is left, with no tag, so no
/// game takes it for the answer to a drop of its own.
#[test]
fn a_part_of_a_heap_leaves_the_rest() {
    let (mut room, ada, bob, mut out) = pair();
    act(
        &mut room,
        ada,
        Act::Drop(thing(HEAP, 1, 3.0, 0.0)),
        (101.0, &mut out),
    );
    out.clear();
    act(
        &mut room,
        bob,
        Act::Take { id: 1, mass: 1.0 },
        (102.0, &mut out),
    );
    let left = lies(&events(&out, ada));
    assert_eq!(
        left.iter().map(|l| (l.mass, l.tag)).collect::<Vec<_>>(),
        [(2.0, 0)]
    );
    assert!(matches!(events(&out, bob).first(), Some(Event::Granted(got)) if got.mass == 1.0));
}

/// A tool put down in a full cube comes back to its player alone, with the tag of its drop.
#[test]
fn a_full_cube_hands_the_thing_back() {
    let (mut room, ada, bob, mut out) = pair();
    for n in 0..CELL_MAX {
        let spaced = thing(TOOL, 1, 1.0, n as f64);
        act(
            &mut room,
            ada,
            Act::Drop(spaced),
            (101.0 + n as f64, &mut out),
        );
    }
    out.clear();
    let late = Lying {
        tag: 9,
        ..thing(TOOL, 1, 1.0, 0.5)
    };
    act(&mut room, ada, Act::Drop(late), (200.0, &mut out));
    assert_eq!(events(&out, ada), [Event::Kept(9)]);
    assert!(events(&out, bob).is_empty());
}

/// Returns what the first player to join `room` at network time `now` is told lies.
fn told_on_joining(room: &mut Room, now: f64) -> Vec<Lying> {
    let mut out = Outbox::default();
    let cyd = join(room, 3, hello("Cyd", 3), now, &mut out);
    lies(&events(&out, cyd))
}

/// A player who joins later finds what lies, and so does a player of a room started again from
/// the world the first one saved, with the ids it had; an empty ground tells nothing.
#[test]
fn the_ground_is_found_later_and_after_a_restart() {
    let (mut room, ada, _, mut out) = pair();
    let heap = Act::Drop(thing(HEAP, 4, 2.0, 0.0));
    act(&mut room, ada, heap, (101.0, &mut out));
    assert_eq!(told_on_joining(&mut room, 102.0).len(), 1);
    assert!(room.changed());
    let config = RoomConfig {
        name: "Test".to_string(),
        password: None,
        cap: 4,
        version: "0.11.0".to_string(),
    };
    let mut again = Room::new(config, room.snapshot(110.0), 0.0);
    let found = told_on_joining(&mut again, 1.0);
    let found: Vec<(u64, f32)> = found.iter().map(|l| (l.id, l.mass)).collect();
    assert_eq!(found, [(1, 2.0)]);
    assert!(told_on_joining(&mut super::kit::room(4, None), 100.0).is_empty());
}
