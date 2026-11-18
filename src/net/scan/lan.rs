//! The games open on the local network, heard from their beacons while the multiplayer menu
//! is open. The socket exists only while the menu does.

use abstractions_net::lan::{Found, Listener};
use abstractions_net::wire::PROTOCOL;
use bevy::prelude::*;

use crate::net::{Lobby, RowStatus, ServerRow};

/// The beacon listener of the open multiplayer menu, or nothing when the port is unusable.
#[derive(Resource, Debug)]
pub struct Scan(Option<Listener>);

/// Returns the row of a game heard on the network: its address, its key and what its beacon
/// says. A beacon carries no round trip, so the ping shows 0.
pub fn row_of(found: &Found) -> ServerRow {
    let beacon = &found.beacon;
    ServerRow {
        name: beacon.name.clone(),
        address: found.addr.to_string(),
        status: RowStatus::Online {
            players: beacon.players,
            cap: beacon.cap,
            ping_ms: 0,
            version: beacon.version.clone(),
            compatible: beacon.protocol == PROTOCOL,
        },
        key: Some(beacon.key),
    }
}

/// Starts listening for beacons as the menu opens.
pub fn listen(mut commands: Commands) {
    let listener = Listener::start()
        .inspect_err(|error| warn!("cannot listen for games on the network: {error}"))
        .ok();
    commands.insert_resource(Scan(listener));
}

/// Stops listening as the menu closes and forgets the games heard.
pub fn deafen(mut commands: Commands, mut lobby: ResMut<Lobby>) {
    commands.remove_resource::<Scan>();
    lobby.lan.clear();
}

/// Reads the beacons that arrived and lists the games heard in the last seconds.
pub fn hear(mut scan: ResMut<Scan>, mut lobby: ResMut<Lobby>) {
    let Some(listener) = scan.0.as_mut() else {
        return;
    };
    let rows: Vec<ServerRow> = listener.poll().iter().map(row_of).collect();
    if lobby.lan != rows {
        lobby.lan = rows;
    }
}
