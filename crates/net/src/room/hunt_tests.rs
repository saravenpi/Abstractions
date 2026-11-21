//! Tests of the hunt the room keeps: blows, kills and who shares the loot, carving, stuck spears
//! and held items, each told to the others and kept for the player who joins later.

use super::hunt_kit::{act, animal, events, hit, kill, spear};
use super::kit::{hello, join};
use super::map_tests::pair;
use crate::host::Outbox;
use crate::log::PlayerId;
use crate::wire::{Act, Event, Hit};

/// A blow reaches the others with the id the server knows the sender by, and a blow on a part
/// that does not exist is dropped.
#[test]
fn a_blow_reaches_the_others_in_the_senders_name() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Hit(hit(40.0)), (101.0, &mut out));
    let wrong = Hit {
        part: 64,
        ..hit(1.0)
    };
    act(&mut room, ada, Act::Hit(wrong), (101.0, &mut out));
    let told = Event::Hit {
        by: PlayerId(1),
        hit: hit(40.0),
    };
    assert_eq!(events(&out, bob), [told]);
    assert!(events(&out, ada).is_empty());
}

/// A kill makes one carcass for everyone, the killer included, credits whoever hurt the animal in
/// the 30 seconds before, keeps the parts broken, and is refused a second time.
#[test]
fn a_kill_makes_one_carcass_and_credits_the_hunters() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Hit(hit(40.0)), (101.0, &mut out));
    let broke = Act::Broke {
        animal: animal(0),
        part: 3,
    };
    act(&mut room, ada, broke, (101.0, &mut out));
    act(&mut room, bob, Act::Hit(hit(60.0)), (140.0, &mut out));
    out.clear();
    act(&mut room, bob, Act::Kill(kill(5_040.0)), (140.0, &mut out));
    act(&mut room, ada, Act::Kill(kill(5_040.0)), (140.0, &mut out));
    let told = events(&out, ada);
    let [Event::Killed(carcass)] = told.as_slice() else {
        panic!("one carcass");
    };
    assert_eq!(events(&out, bob), events(&out, ada));
    assert_eq!(
        (carcass.since, carcass.broken, carcass.taken),
        (5_040.0, 8, 0)
    );
    assert_eq!(carcass.credited, [PlayerId(2)]);
}

/// A kill with a time far from the clock of the server is taken at the clock of the server.
#[test]
fn a_kill_far_from_the_clock_is_taken_at_the_clock() {
    let (mut room, ada, _, mut out) = pair();
    act(&mut room, ada, Act::Kill(kill(9_999.0)), (110.0, &mut out));
    let told = events(&out, ada);
    let [Event::Killed(carcass)] = told.as_slice() else {
        panic!("one carcass");
    };
    assert_eq!(carcass.since, 5_010.0);
}

/// A part is taken from a carcass once: two players cut it at once, the first is granted it, the
/// second is refused, and the others hear the cut once.
#[test]
fn a_part_is_taken_once() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Kill(kill(5_001.0)), (101.0, &mut out));
    out.clear();
    let carve = Act::Carve {
        animal: animal(0),
        part: 4,
    };
    act(&mut room, ada, carve.clone(), (102.0, &mut out));
    act(&mut room, bob, carve, (102.0, &mut out));
    let carved = Event::Carved {
        animal: animal(0),
        part: 4,
        by: PlayerId(1),
    };
    let answer = |granted| Event::Cut {
        animal: animal(0),
        part: 4,
        granted,
    };
    assert_eq!(events(&out, bob), [carved, answer(false)]);
    assert_eq!(events(&out, ada), [answer(true)]);
}

/// A player who joins later is told the carcass with its taken parts, the spears that stand
/// and what the others hold, and nothing at all when there is nothing.
#[test]
fn a_later_player_finds_the_hunt() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Kill(kill(5_001.0)), (101.0, &mut out));
    let carve = Act::Carve {
        animal: animal(0),
        part: 1,
    };
    act(&mut room, ada, carve, (101.0, &mut out));
    act(&mut room, bob, Act::Stick(spear(7)), (101.0, &mut out));
    act(
        &mut room,
        bob,
        Act::Held(Some(vec![3; 20])),
        (101.0, &mut out),
    );
    let mut later = Outbox::default();
    let cyd = join(&mut room, 3, hello("Cyd", 3), 102.0, &mut later);
    let told = events(&later, cyd);
    let [Event::World(world)] = told.as_slice() else {
        panic!("the hunt");
    };
    assert_eq!((world.carcasses.len(), world.carcasses[0].taken), (1, 2));
    assert_eq!(world.spears[0].key.owner, PlayerId(2));
    assert_eq!(world.held, [(PlayerId(2), vec![3; 20])]);
    let mut empty = Outbox::default();
    let mut fresh = super::kit::room(16, None);
    let dan = join(&mut fresh, 1, hello("Dan", 4), 100.0, &mut empty);
    assert!(events(&empty, dan).is_empty());
}
