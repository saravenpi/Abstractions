//! The blows of this machine's robot on animals, and the parts they break, sent to the server.

use abstractions_net::wire::{self, Act, AnimalKey, ClientMsg};
use bevy::prelude::*;

use super::Hunted;
use crate::avatar::strike::Hit;
use crate::craft::build::Intent;
use crate::fauna::{Beast, Creature};
use crate::net::link::Link;
use crate::space::Location;

/// Frames an animal that was struck is watched for a part that breaks: the blow is read before
/// the fauna harms the animal, so its parts break a frame or two later.
const WATCH: u8 = 3;

/// Returns the key of the animal `creature` on the body the player is at.
pub fn key_of(location: &Location, creature: &Creature) -> AnimalKey {
    AnimalKey {
        body: location.body,
        herd: creature.herd,
        slot: creature.slot,
    }
}

/// Tells the server of each blow that landed on a part of an animal, and watches the animal for a
/// part that breaks.
pub fn hits(
    mut blows: MessageReader<Hit>,
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    location: Option<Res<Location>>,
    animals: Query<(&Creature, &Beast)>,
) {
    let Some(location) = location else {
        return;
    };
    for hit in blows.read() {
        let Ok((creature, beast)) = animals.get(hit.victim) else {
            continue;
        };
        let Some(part) = beast.parts.iter().position(|p| *p == Some(hit.part)) else {
            continue;
        };
        let blow = wire::Hit {
            animal: key_of(&location, creature),
            part: part as u8,
            joules: hit.blow.energy * hit.outcome.harm(),
            blunt: hit.intent == Intent::Crush,
        };
        link.send(ClientMsg::Hunt(Act::Hit(blow)));
        hunted.victims.push((hit.victim, WATCH));
    }
}

/// Tells the server of each part that broke on an animal that was struck lately.
pub fn breaks(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    location: Option<Res<Location>>,
    animals: Query<(&Creature, &Beast)>,
) {
    let Some(location) = location else {
        hunted.victims.clear();
        return;
    };
    let victims = std::mem::take(&mut hunted.victims);
    for (victim, left) in victims {
        let Ok((creature, beast)) = animals.get(victim) else {
            continue;
        };
        let animal = key_of(&location, creature);
        let sent = hunted.broken.entry(animal).or_insert(0);
        let fresh = beast.wits.broken.0 & !*sent;
        *sent |= fresh;
        for part in (0..64u8).filter(|part| fresh >> part & 1 == 1) {
            link.send(ClientMsg::Hunt(Act::Broke { animal, part }));
        }
        if left > 1 {
            hunted.victims.push((victim, left - 1));
        }
    }
}
