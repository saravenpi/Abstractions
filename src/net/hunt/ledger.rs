//! The kills of this machine's robot and the parts it cut from carcasses, sent to the server.
//!
//! A kill is read from the carcass the fauna lays when an animal dies, and a cut from the parts
//! taken of that carcass, so the fauna needs no change. What the server told is already in
//! [`Hunted`], and is not sent back.

use abstractions_net::wire::{Act, AnimalKey, ClientMsg, Kill};
use bevy::prelude::*;

use super::Hunted;
use crate::fauna::{Herds, carcass_key};
use crate::life::hunt::{Carcass, RECOVERY};
use crate::net::link::Link;
use crate::space::Location;

/// Returns the key of the animal of `carcass` on the body the player is at.
pub fn animal(location: &Location, carcass: &Carcass) -> AnimalKey {
    AnimalKey {
        body: location.body,
        herd: carcass.herd,
        slot: carcass.slot,
    }
}

/// Tells the server of each kill of this machine it does not know yet.
pub fn kills(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    herds: Res<Herds>,
    location: Option<Res<Location>>,
) {
    let Some(location) = location else {
        return;
    };
    for carcass in &herds.slain {
        let key = carcass_key(carcass);
        if !hunted.known.insert(key) {
            continue;
        }
        let grade = herds.carved.iter().find(|c| c.key == key);
        let lasts = (carcass.bones - carcass.since).max(RECOVERY);
        link.send(ClientMsg::Hunt(Act::Kill(Kill {
            animal: animal(&location, carcass),
            species: carcass.species,
            at: carcass.at.to_array(),
            since: carcass.since,
            grade: grade.map_or(2, |c| c.kill as u8),
            lasts: lasts as f32,
        })));
    }
}

/// Tells the server of each part cut from a carcass that it does not know of yet.
pub fn carves(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    herds: Res<Herds>,
    location: Option<Res<Location>>,
) {
    let Some(location) = location else {
        return;
    };
    for cut in &herds.carved {
        let sent = hunted.taken.entry(cut.key).or_insert(0);
        let fresh = cut.taken & !*sent;
        *sent |= fresh;
        let found = herds.slain.iter().find(|c| carcass_key(c) == cut.key);
        let Some(carcass) = found.filter(|_| fresh != 0) else {
            continue;
        };
        for part in (0..64u8).filter(|part| fresh >> part & 1 == 1) {
            let animal = animal(&location, carcass);
            link.send(ClientMsg::Hunt(Act::Carve { animal, part }));
        }
    }
}
