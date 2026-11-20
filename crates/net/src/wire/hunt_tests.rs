//! Tests of the hunting messages: every one survives the wire at its documented size, the
//! numbers of the old messages stay, and bytes of the old format are refused.

use super::body_tests::planet;
use super::{Act, AnimalKey, Carcass, ClientMsg, Event, Hit, HuntWorld, ITEM_MAX, Kill};
use super::{ServerMsg, Spear, SpearKey, decode, encode};
use crate::log::PlayerId;

/// Returns an animal whose herd id takes all ten bytes of a varint, the worst case.
pub(super) fn animal() -> AnimalKey {
    AnimalKey {
        body: planet(),
        herd: u64::MAX,
        slot: 7,
    }
}

/// Returns the kill of that animal.
fn kill() -> Kill {
    Kill {
        animal: animal(),
        species: 300,
        at: [0.6, -0.48, 0.64],
        since: 123_456_789.123,
        grade: 1,
        lasts: 2700.0,
    }
}

/// Returns a carcass with the most bytes: every part taken and broken, three credited players.
fn carcass() -> Carcass {
    Carcass {
        animal: animal(),
        species: 300,
        at: [0.6, -0.48, 0.64],
        since: 123_456_789.123,
        grade: 2,
        lasts: 2700.0,
        taken: u64::MAX,
        broken: u64::MAX,
        credited: vec![PlayerId(1), PlayerId(2), PlayerId(300)],
    }
}

/// Returns a spear stuck in an animal, carrying an item of the largest size.
fn spear() -> Spear {
    Spear {
        key: SpearKey {
            owner: PlayerId(3),
            tag: 70_000,
        },
        animal: animal(),
        at: [0.1, 1.2, -0.3],
        dir: [0.0, 1.0, 0.0],
        item: vec![9; ITEM_MAX],
    }
}

/// Returns the length of `message` on a stream, and checks it comes back as it was sent.
pub(super) fn size<T>(message: &T) -> usize
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let bytes = encode(message).unwrap();
    let (got, used) = decode::<T>(&bytes, bytes.len()).unwrap().unwrap();
    assert_eq!((&got, used), (message, bytes.len()));
    bytes.len()
}

/// Every act of a player comes back, and a hit, a kill, a carve and a spear stay within their
/// documented sizes on a stream, length prefix included.
#[test]
fn every_act_comes_back_at_its_size() {
    let hit = Hit {
        animal: animal(),
        part: 5,
        joules: 93.5,
        blunt: true,
    };
    assert!(size(&ClientMsg::Hunt(Act::Hit(hit))) <= 40);
    assert!(size(&ClientMsg::Hunt(Act::Kill(kill()))) <= 80);
    let carve = Act::Carve {
        animal: animal(),
        part: 63,
    };
    assert!(size(&ClientMsg::Hunt(carve)) <= 36);
    assert!(size(&ClientMsg::Hunt(Act::Stick(spear()))) <= ITEM_MAX + 60);
    size(&ClientMsg::Hunt(Act::Pull(spear().key)));
    size(&ClientMsg::Hunt(Act::Held(Some(vec![1; ITEM_MAX]))));
    size(&ClientMsg::Hunt(Act::Held(None)));
    let broke = Act::Broke {
        animal: animal(),
        part: 2,
    };
    size(&ClientMsg::Hunt(broke));
}

/// Every event of a server comes back, and a carcass stays within its documented size.
#[test]
fn every_event_comes_back_at_its_size() {
    let hit = Hit {
        animal: animal(),
        part: 5,
        joules: 93.5,
        blunt: false,
    };
    size(&ServerMsg::Hunt(Event::Hit {
        by: PlayerId(2),
        hit,
    }));
    assert!(size(&ServerMsg::Hunt(Event::Killed(carcass()))) <= 110);
    let carved = Event::Carved {
        animal: animal(),
        part: 4,
        by: PlayerId(2),
    };
    size(&ServerMsg::Hunt(carved));
    size(&ServerMsg::Hunt(Event::Stuck(spear())));
}

/// Every other event of a server comes back, the state of the hunt after joining included.
#[test]
fn the_other_events_come_back() {
    let pulled = Event::Pulled {
        key: spear().key,
        by: PlayerId(9),
    };
    size(&ServerMsg::Hunt(pulled));
    let held = Event::Held {
        by: PlayerId(2),
        item: Some(vec![4; 70]),
    };
    size(&ServerMsg::Hunt(held));
    let other = Spear {
        key: SpearKey {
            owner: PlayerId(4),
            tag: 1,
        },
        item: vec![2; 10],
        ..spear()
    };
    let world = HuntWorld {
        carcasses: vec![carcass()],
        spears: vec![spear(), other],
        held: vec![(PlayerId(2), vec![1; 70])],
        ..HuntWorld::default()
    };
    size(&ServerMsg::Hunt(Event::World(world)));
}
