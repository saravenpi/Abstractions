//! The answers of the server to the cuts of this player: a part granted goes into the pack, a part
//! another player cut first gives no tissue and the line "Already taken". Each cut waits in
//! [`Claims`] from the press of the key to the answer.

use abstractions_net::wire::{AnimalKey, Event as Told, ServerMsg};
use bevy::prelude::*;

use super::ledger;
use crate::craft::ground::{Note, TAKEN};
use crate::craft::harvest::{Claims, Harvest};
use crate::fauna::{Herds, carcass_key};
use crate::net::link::{FromServer, Link, Receive};
use crate::space::Location;

/// Returns true for the answer of the server to a cut, which this module takes.
pub(super) fn is_answer(event: &Told) -> bool {
    matches!(event, Told::Cut { .. })
}

/// Returns the key of the carcass of `animal` this machine holds on the body of `location`.
fn key_of(herds: &Herds, location: &Location, animal: &AnimalKey) -> Option<u64> {
    let found = herds
        .slain
        .iter()
        .find(|c| ledger::animal(location, c) == *animal);
    found.map(carcass_key)
}

/// What answering a cut reads and writes.
type Answers<'w> = (ResMut<'w, Claims>, Res<'w, Herds>, Res<'w, Location>);

/// Harvests each cut the server granted, and says in the banner which it refused.
fn answer(
    mut from: MessageReader<FromServer>,
    (mut claims, herds, location): Answers,
    (mut asks, mut notes): (MessageWriter<Harvest>, MessageWriter<Note>),
) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Hunt(Told::Cut {
            animal,
            part,
            granted,
        }) = msg
        else {
            continue;
        };
        let key = key_of(&herds, &location, animal);
        let Some(cut) = key.and_then(|key| claims.answered((key, *part))) else {
            continue;
        };
        match *granted {
            true => _ = asks.write(Harvest(cut)),
            false => _ = notes.write(Note(TAKEN.to_string())),
        }
    }
}

/// Lets the server decide who gets each part once a link opens.
fn hand_over(mut claims: ResMut<Claims>) {
    claims.remote = true;
}

/// Harvests the cuts the server never answered once the link is gone, and decides alone again.
fn take_back(mut claims: ResMut<Claims>, mut asks: MessageWriter<Harvest>) {
    for cut in claims.release() {
        asks.write(Harvest(cut));
    }
}

/// Registers the answers to the cuts. Every system waits for a link, or for its end.
pub(super) fn plugin(app: &mut App) {
    let ready = resource_exists::<Herds>.and_then(resource_exists::<Location>);
    app.init_resource::<Claims>();
    app.add_systems(
        Update,
        (
            hand_over.run_if(resource_added::<Link>),
            answer
                .after(Receive)
                .run_if(resource_exists::<Link>)
                .run_if(on_message::<FromServer>)
                .run_if(ready),
            take_back.run_if(resource_removed::<Link>),
        )
            .chain(),
    );
}
