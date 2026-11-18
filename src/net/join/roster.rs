//! The welcome that starts the server's world, the players who come and go, and the key of a
//! server seen the first time, shown once the player is in the world.

use abstractions_net::wire::{PlayerInfo, ServerMsg};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::world_of;
use crate::net::link::{FreshKey, FromServer, Link, Role};
use crate::net::{Lobby, PlayerJoined, PlayerLeft, Players, Session, SessionNotice};
use crate::saves::{Starter, Unsaved};

/// What a welcome changes.
#[derive(SystemParam)]
pub struct Arrival<'w, 's> {
    /// Starts the server's world.
    starter: Starter<'w, 's>,
    /// Marks that world as one never written to the local saves folder.
    commands: Commands<'w, 's>,
    /// Where the game stands with other players.
    session: ResMut<'w, Session>,
    /// The players of the session.
    players: ResMut<'w, Players>,
    /// The name each server gives itself, from the last status request.
    lobby: Res<'w, Lobby>,
}

/// Returns the players of a welcome as the interface lists them.
fn listed(players: &[PlayerInfo]) -> Vec<(u32, String)> {
    players
        .iter()
        .map(|player| (player.id.0, player.name.clone()))
        .collect()
}

/// Takes the welcome: the id and the players, then, when joining, the server's world and
/// [`Session::Joined`]. A host keeps the world on screen.
pub fn welcome(mut from: MessageReader<FromServer>, mut link: ResMut<Link>, mut arrival: Arrival) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Welcome(welcome) = msg else {
            continue;
        };
        link.you = Some(welcome.you);
        arrival.players.0 = listed(&welcome.players);
        let Role::Join { address, .. } = &link.role else {
            continue;
        };
        let named = arrival.lobby.find(address).map(|row| row.name.trim());
        let name = named
            .filter(|name| !name.is_empty())
            .unwrap_or(address.as_str())
            .to_string();
        arrival.starter.start(world_of(welcome, &name));
        arrival.commands.insert_resource(Unsaved);
        *arrival.session = Session::Joined {
            address: address.clone(),
            name,
        };
    }
}

/// The banner lines of the players who come and go.
type Presence<'w> = (
    MessageWriter<'w, PlayerJoined>,
    MessageWriter<'w, PlayerLeft>,
);

/// Keeps the list of players and tells the banner who joined and who left.
pub fn roster(
    mut from: MessageReader<FromServer>,
    mut players: ResMut<Players>,
    (mut joined, mut left): Presence,
) {
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Joined(info) => {
                players.0.retain(|(id, _)| *id != info.id.0);
                players.0.push((info.id.0, info.name.clone()));
                joined.write(PlayerJoined {
                    name: info.name.clone(),
                });
            }
            ServerMsg::Left(gone) => {
                let Some(at) = players.0.iter().position(|(id, _)| *id == gone.0) else {
                    continue;
                };
                let (_, name) = players.0.remove(at);
                left.write(PlayerLeft { name });
            }
            _ => {}
        }
    }
}

/// Run condition: the key of a server seen the first time waits for the banner.
pub fn pending(fresh: Option<Res<FreshKey>>) -> bool {
    fresh.is_some_and(|fresh| fresh.0.is_some())
}

/// Shows the key of a server seen the first time in the banner, once the player is in the
/// world.
pub fn notify(mut fresh: ResMut<FreshKey>, mut notices: MessageWriter<SessionNotice>) {
    if let Some(line) = fresh.0.take() {
        notices.write(SessionNotice { line });
    }
}
