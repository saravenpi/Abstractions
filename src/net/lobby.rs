//! What the multiplayer menu lists: the saved servers with their status, the games found on the
//! local network, and the players of the session.

use bevy::prelude::*;

/// What a server answered to the last status request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowStatus {
    /// The request is on its way.
    Pinging,
    /// The server answered.
    Online {
        /// Players connected now.
        players: u8,
        /// Most players the server takes.
        cap: u8,
        /// Round trip of the status request, in milliseconds.
        ping_ms: u32,
        /// The game version the server runs, as `0.9.0`.
        version: String,
        /// Whether this game can join it.
        compatible: bool,
    },
    /// The server did not answer.
    Offline,
    /// The server shows another key than the one remembered for its address.
    KeyChanged,
}

/// One server of a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerRow {
    /// The name the server gives itself.
    pub name: String,
    /// Where to reach it: a domain or an IP, with an optional port.
    pub address: String,
    /// What it answered last.
    pub status: RowStatus,
    /// The fingerprint a LAN beacon announced. Saved servers have none here.
    pub key: Option<[u8; 32]>,
}

/// The servers the multiplayer menu shows, filled by the network.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct Lobby {
    /// Status of the saved servers, matched to the settings by address.
    pub saved: Vec<ServerRow>,
    /// Games found on the local network.
    pub lan: Vec<ServerRow>,
}

impl Lobby {
    /// Returns the row of `address`, saved or found on the network.
    pub fn find(&self, address: &str) -> Option<&ServerRow> {
        self.saved
            .iter()
            .chain(&self.lan)
            .find(|row| row.address == address)
    }
}

/// The players of the session: an id and a name each.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct Players(pub Vec<(u32, String)>);
