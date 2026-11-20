//! Tests of the field acts the room relays and keeps: blows on robots, wounds, impacts, attacks
//! and taken rocks.

use super::hunt_kit::{act, animal, events};
use super::kit::{hello, join};
use super::map_tests::pair;
use crate::host::Outbox;
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{Act, Attack, Blow, Event, Impact, Rock, Wound};

/// Returns a blow on player 2.
fn blow(victim: u32) -> Blow {
    Blow {
        victim: PlayerId(victim),
        at: [0.0, 1.0, 0.0],
        joules: 30.0,
        cut: true,
        dir: [0.0, 0.0, -1.0],
        momentum: 5.0,
    }
}

/// A blow on a robot reaches its player alone, in the name of the sender; a blow on oneself, or
/// on nobody, reaches no one.
#[test]
fn a_blow_reaches_only_its_victim() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Blow(blow(2)), (101.0, &mut out));
    act(&mut room, ada, Act::Blow(blow(1)), (101.0, &mut out));
    act(&mut room, ada, Act::Blow(blow(9)), (101.0, &mut out));
    let told = Event::Blow {
        by: PlayerId(1),
        blow: blow(2),
    };
    assert_eq!(events(&out, bob), [told]);
    assert!(events(&out, ada).is_empty());
}

/// Opens a wound on member 0 and plugs it twice, and takes a rock twice, as Ada and Bob.
fn field() -> (
    super::Room,
    crate::host::ConnId,
    crate::host::ConnId,
    Outbox,
    Rock,
) {
    let (mut room, ada, bob, mut out) = pair();
    let wound = Wound {
        part: 2,
        depth: 0.05,
        width: 0.02,
        vessel: 1,
        opened: 5_001.0,
        plugged: true,
    };
    let open = Act::Wound {
        animal: animal(),
        wound,
    };
    act(&mut room, ada, open, (101.0, &mut out));
    let plug = Act::Plug {
        animal: animal(),
        part: 2,
    };
    act(&mut room, ada, plug.clone(), (101.0, &mut out));
    act(&mut room, ada, plug, (101.0, &mut out));
    let rock = Rock {
        planet: BodyId::default(),
        id: 77,
    };
    act(&mut room, ada, Act::Rock(rock), (101.0, &mut out));
    act(&mut room, bob, Act::Rock(rock), (101.0, &mut out));
    (room, ada, bob, out, rock)
}

/// A wound, its plug and a rock are told to the others once each, and a rock taken twice is not
/// told back.
#[test]
fn wounds_and_rocks_are_told_once() {
    let (_, ada, bob, out, _) = field();
    assert_eq!(events(&out, bob).len(), 3);
    assert!(events(&out, ada).is_empty());
}

/// A player who joins later is told the wound with its plug and the rock taken.
#[test]
fn a_later_player_finds_the_wounds_and_rocks() {
    let (mut room, _, _, _, rock) = field();
    let mut later = Outbox::default();
    let cyd = join(&mut room, 3, hello("Cyd", 3), 102.0, &mut later);
    let seen = events(&later, cyd);
    let [Event::World(world)] = seen.as_slice() else {
        panic!("the world");
    };
    assert_eq!(world.rocks, [rock]);
    assert!(world.wounds.len() == 1 && world.wounds[0].1.plugged);
}

/// An impact and an attack go to the others and are not kept.
#[test]
fn impacts_and_attacks_are_relayed_not_kept() {
    let (mut room, ada, bob, mut out) = pair();
    let impact = Impact {
        body: BodyId::default(),
        at: [1.0, 2.0, 3.0],
        joules: 9.0,
        cut: false,
        outer: 1,
        tool: 2,
        result: 0,
    };
    let attack = Attack {
        animal: animal(),
        kind: 1,
        target: Some(PlayerId(2)),
        started: 5_001.0,
    };
    act(&mut room, ada, Act::Impact(impact), (101.0, &mut out));
    act(&mut room, ada, Act::Attack(attack), (101.0, &mut out));
    assert_eq!(
        events(&out, bob),
        [Event::Impact(impact), Event::Attack(attack)]
    );
    assert!(room.snapshot(110.0).hunt.wounds.is_empty());
}
