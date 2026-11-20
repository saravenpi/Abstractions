//! Tests of what the room keeps of the hunt: spears, carcasses that go, limits and saves.

use super::Room;
use super::hunt_kit::{act, animal, events, hit, kill, spear};
use super::kit::{hello, join};
use super::map_tests::pair;
use crate::host::{Host, Outbox};
use crate::log::PlayerId;
use crate::wire::{Act, Event, HuntWorld, Spear, SpearKey};

/// A spear is stamped with its sender, stands until pulled, and a second pull is silent.
#[test]
fn a_spear_stands_until_it_is_pulled() {
    let (mut room, ada, bob, mut out) = pair();
    act(&mut room, ada, Act::Stick(spear(7)), (101.0, &mut out));
    let stuck = Spear {
        key: SpearKey {
            owner: PlayerId(1),
            tag: 7,
        },
        ..spear(7)
    };
    assert_eq!(events(&out, bob), [Event::Stuck(stuck.clone())]);
    out.clear();
    act(&mut room, bob, Act::Pull(stuck.key), (102.0, &mut out));
    act(&mut room, bob, Act::Pull(stuck.key), (102.0, &mut out));
    let pulled = Event::Pulled {
        key: stuck.key,
        by: PlayerId(2),
    };
    assert_eq!(events(&out, ada), [pulled]);
    let long = Spear {
        item: vec![0; 97],
        ..spear(8)
    };
    out.clear();
    act(&mut room, ada, Act::Stick(long), (103.0, &mut out));
    assert!(events(&out, bob).is_empty());
}

/// A carcass goes once its time is up, the spear in it with it, and the room says whether
/// something changed since it was last saved.
#[test]
fn a_carcass_goes_when_its_time_is_up() {
    let (mut room, ada, _, mut out) = pair();
    assert!(!room.changed());
    act(&mut room, ada, Act::Kill(kill(5_001.0)), (101.0, &mut out));
    act(&mut room, ada, Act::Stick(spear(1)), (101.0, &mut out));
    assert!(room.changed());
    assert_eq!(room.snapshot(150.0).hunt.spears.len(), 1);
    room.saved();
    assert!(!room.changed());
    room.tick(210.0, &mut out);
    assert_eq!(room.snapshot(210.0).hunt, HuntWorld::default());
    assert!(room.changed());
}

/// A player may send 30 acts a second; the rest are dropped.
#[test]
fn acts_are_limited_per_second() {
    let (mut room, ada, bob, mut out) = pair();
    for n in 0..40 {
        act(&mut room, ada, Act::Hit(hit(n as f32)), (101.0, &mut out));
    }
    assert_eq!(events(&out, bob).len(), 30);
}

/// A server that restarts serves the carcasses of the world it saved.
#[test]
fn a_saved_hunt_is_served_again() {
    let (mut room, ada, _, mut out) = pair();
    act(&mut room, ada, Act::Kill(kill(5_001.0)), (101.0, &mut out));
    let world = room.snapshot(110.0);
    let mut again = Room::new(
        super::RoomConfig {
            name: "Test".to_string(),
            password: None,
            cap: 4,
            version: "0.9.0".to_string(),
        },
        world,
        0.0,
    );
    let mut later = Outbox::default();
    let bob = join(&mut again, 1, hello("Bob", 2), 1.0, &mut later);
    let told = events(&later, bob);
    let [Event::World(world)] = told.as_slice() else {
        panic!("the hunt");
    };
    assert_eq!(world.carcasses[0].animal, animal(0));
}
