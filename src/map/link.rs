//! The map on a server: the active pin of every player and the pins of this one.
//!
//! This player's active pin goes out as their waypoint whenever it changes, and the waypoints
//! of the others come in with the welcome and with each change. The pins ride in the record
//! the server keeps of the player, as TOML text.

use abstractions_net::wire::{ClientMsg, ServerMsg, Waypoint as Wire};
use bevy::math::DVec3;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::places::Places;
use crate::net::{FromServer, Link};
use crate::saves::Pin;

/// Sent whenever the active pin of this player changes, for the server: the pin, or nothing
/// when none is active.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct WaypointChanged(pub Option<Pin>);

/// The pins as the record of a player holds them.
#[derive(Serialize, Deserialize, Default)]
struct Marks {
    /// Every pin, in the order they were dropped.
    #[serde(default)]
    mark: Vec<Pin>,
}

/// Returns the pins as the TOML text of the record of a player, empty for none.
pub fn marks_text(pins: &[Pin]) -> String {
    let marks = Marks {
        mark: pins.to_vec(),
    };
    let text = toml::to_string(&marks).ok().filter(|_| !pins.is_empty());
    text.unwrap_or_default()
}

/// Returns the pins written in the record of a player, none when it holds none.
pub fn marks_of(text: &str) -> Vec<Pin> {
    toml::from_str::<Marks>(text).map_or_else(|_| Vec::new(), |marks| marks.mark)
}

/// Returns a pin as the server sends it.
pub fn to_wire(pin: &Pin) -> Wire {
    Wire {
        body: pin.body,
        dir: DVec3::from_array(pin.dir).as_vec3().to_array(),
        name: pin.name.clone(),
    }
}

/// Returns a pin the server sent, or nothing when its direction is not one.
pub fn from_wire(wire: &Wire) -> Option<Pin> {
    let dir = DVec3::from_array(wire.dir.map(f64::from)).try_normalize()?;
    Some(Pin {
        name: wire.name.clone(),
        body: wire.body,
        dir: dir.to_array(),
    })
}

/// Sends every change of the active pin of this player to the server.
pub(super) fn send(mut changes: MessageReader<WaypointChanged>, mut link: ResMut<Link>) {
    for WaypointChanged(pin) in changes.read() {
        link.send(ClientMsg::Waypoint(pin.as_ref().map(to_wire)));
    }
}

/// Takes in the active pins of the other players: all of them with the welcome, then each one
/// that is set or cleared. This player's id comes from the link and picks its colour. The
/// active pin this player kept from before goes to the server with the welcome, which knows
/// none of it.
pub(super) fn receive(
    mut read: MessageReader<FromServer>,
    mut link: ResMut<Link>,
    mut places: ResMut<Places>,
) {
    let you = link.you.map(|you| you.0);
    if let Some(you) = you.filter(|you| *you != places.me) {
        places.me = you;
    }
    for FromServer(message) in read.read() {
        match message {
            ServerMsg::Welcome(welcome) => {
                let given = welcome.waypoints.iter();
                let friends = given.filter_map(|(player, wire)| Some((player.0, from_wire(wire)?)));
                places.friends = friends.filter(|(player, _)| Some(*player) != you).collect();
                if let Some(mine) = places.active() {
                    link.send(ClientMsg::Waypoint(Some(to_wire(mine))));
                }
            }
            ServerMsg::Waypoint { player, waypoint } if Some(player.0) != you => {
                match waypoint.as_ref().and_then(from_wire) {
                    Some(set) => places.friends.insert(player.0, set),
                    None => places.friends.remove(&player.0),
                };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
