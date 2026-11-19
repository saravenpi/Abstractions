//! The map on a server: the waypoints of every player and the favourites of this one.
//!
//! This player's waypoint goes out whenever it changes, and the waypoints of the others come
//! in with the welcome and with each change. The favourites ride in the record the server
//! keeps of the player, as TOML text.

use abstractions_net::wire::{ClientMsg, ServerMsg, Waypoint as Wire};
use bevy::math::DVec3;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::waypoint::{Waypoint, WaypointChanged, Waypoints};
use crate::net::{FromServer, Link};
use crate::saves::Favourite;

/// The favourites as the record of a player holds them.
#[derive(Serialize, Deserialize, Default)]
struct Marks {
    /// Every favourite, in the order they were saved.
    #[serde(default)]
    mark: Vec<Favourite>,
}

/// Returns the favourites as the TOML text of the record of a player, empty for none.
pub fn marks_text(favourites: &[Favourite]) -> String {
    let marks = Marks {
        mark: favourites.to_vec(),
    };
    let text = toml::to_string(&marks)
        .ok()
        .filter(|_| !favourites.is_empty());
    text.unwrap_or_default()
}

/// Returns the favourites written in the record of a player, none when it holds none.
pub fn marks_of(text: &str) -> Vec<Favourite> {
    toml::from_str::<Marks>(text).map_or_else(|_| Vec::new(), |marks| marks.mark)
}

/// Returns a waypoint as the server sends it.
pub fn to_wire(waypoint: &Waypoint) -> Wire {
    Wire {
        body: waypoint.body,
        dir: waypoint.dir.as_vec3().to_array(),
        name: waypoint.name.clone(),
    }
}

/// Returns a waypoint the server sent, or nothing when its direction is not one.
pub fn from_wire(wire: &Wire) -> Option<Waypoint> {
    let dir = DVec3::from_array(wire.dir.map(f64::from)).try_normalize()?;
    Some(Waypoint {
        body: wire.body,
        dir,
        name: wire.name.clone(),
    })
}

/// Sends every change of this player's waypoint to the server.
pub(super) fn send(mut changes: MessageReader<WaypointChanged>, mut link: ResMut<Link>) {
    for WaypointChanged(waypoint) in changes.read() {
        link.send(ClientMsg::Waypoint(waypoint.as_ref().map(to_wire)));
    }
}

/// Takes in the waypoints of the other players: all of them with the welcome, then each one
/// that is set or cleared. This player's id comes from the link and picks its colour. A
/// waypoint this player kept from before goes to the server with the welcome, which knows
/// none of it.
pub(super) fn receive(
    mut read: MessageReader<FromServer>,
    mut link: ResMut<Link>,
    mut waypoints: ResMut<Waypoints>,
) {
    let you = link.you.map(|you| you.0);
    if let Some(you) = you.filter(|you| *you != waypoints.me) {
        waypoints.me = you;
    }
    for FromServer(message) in read.read() {
        match message {
            ServerMsg::Welcome(welcome) => {
                let given = welcome.waypoints.iter();
                let friends = given.filter_map(|(player, wire)| Some((player.0, from_wire(wire)?)));
                waypoints.friends = friends.filter(|(player, _)| Some(*player) != you).collect();
                if let Some(mine) = &waypoints.mine {
                    link.send(ClientMsg::Waypoint(Some(to_wire(mine))));
                }
            }
            ServerMsg::Waypoint { player, waypoint } if Some(player.0) != you => {
                match waypoint.as_ref().and_then(from_wire) {
                    Some(set) => waypoints.friends.insert(player.0, set),
                    None => waypoints.friends.remove(&player.0),
                };
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
