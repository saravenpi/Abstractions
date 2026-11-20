//! The sound and the chips of a blow, for everyone: an impact of this machine goes out, and one that
//! comes back is sent to the audio as the one of this machine is.

use bevy::prelude::*;

use super::codec::{from_wire, to_wire};
use crate::audio::Impact;
use crate::net::hunt::Hunted;
use crate::net::hunt::hooks::{self, Echoes, keep};
use crate::net::link::Link;
use crate::space::{Anchor, Location};

/// Tells the server of each impact of this machine, but not of the ones it played for the others.
pub fn send(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    (anchor, location): (Res<Anchor>, Res<Location>),
    mut told: MessageReader<Impact>,
) {
    for impact in told.read() {
        if let Some(at) = hunted.played.impacts.iter().position(|p| p == impact) {
            hunted.played.impacts.swap_remove(at);
        } else if let Some(wire) = to_wire(impact, location.body, &anchor) {
            hooks::send_impact(&mut link, wire);
        }
    }
}

/// Plays the impacts of the others that landed on the body of this player.
pub fn play(
    (mut echoes, mut hunted): (ResMut<Echoes>, ResMut<Hunted>),
    (anchor, location): (Res<Anchor>, Res<Location>),
    mut sounds: MessageWriter<Impact>,
) {
    for told in std::mem::take(&mut echoes.impacts) {
        if let Some(impact) = from_wire(&told, location.body, &anchor) {
            keep(&mut hunted.played.impacts, impact);
            sounds.write(impact);
        }
    }
}
