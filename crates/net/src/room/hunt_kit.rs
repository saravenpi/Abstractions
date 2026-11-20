//! Helpers the hunt tests share: an animal, blows, kills, spears and what a connection hears.

use super::Room;
use super::kit::messages;
use crate::host::{ConnId, Host, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{Act, AnimalKey, ClientMsg, Event, Hit, Kill, ServerMsg, Spear, SpearKey};

/// Returns member `slot` of herd 9 on the default body.
pub(super) fn animal(slot: u8) -> AnimalKey {
    AnimalKey {
        body: BodyId::default(),
        herd: 9,
        slot,
    }
}

/// Returns a blow of `joules` on part 2 of member 0.
pub(super) fn hit(joules: f32) -> Hit {
    Hit {
        animal: animal(0),
        part: 2,
        joules,
        blunt: false,
    }
}

/// Returns the kill of member 0 at universe time `since`.
pub(super) fn kill(since: f64) -> Kill {
    Kill {
        animal: animal(0),
        species: 4,
        at: [0.0, 1.0, 0.0],
        since,
        grade: 0,
        lasts: 100.0,
    }
}

/// Sends `act` from `conn` at network time `now`.
pub(super) fn act(room: &mut Room, conn: ConnId, act: Act, (now, out): (f64, &mut Outbox)) {
    room.message(conn, ClientMsg::Hunt(act), now, out);
}

/// Returns the hunting events queued for `conn`.
pub(super) fn events(out: &Outbox, conn: ConnId) -> Vec<Event> {
    let all = messages(out, conn).into_iter();
    all.filter_map(|msg| match msg {
        ServerMsg::Hunt(event) => Some(event),
        _ => None,
    })
    .collect()
}

/// Returns a spear of tag `tag` that claims to be Bob's, stuck in member 0.
pub(super) fn spear(tag: u32) -> Spear {
    Spear {
        key: SpearKey {
            owner: PlayerId(2),
            tag,
        },
        animal: animal(0),
        at: [1.0, 2.0, 3.0],
        dir: [0.0, 1.0, 0.0],
        item: vec![5; 40],
    }
}
