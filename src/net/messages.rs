//! Requests the interface sends to the network, and what the network tells the interface.

use bevy::prelude::*;

/// Join the server at `address`.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct JoinServer {
    /// The address typed or picked.
    pub address: String,
    /// The fingerprint the server must show, when a LAN beacon announced it.
    pub expected_key: Option<[u8; 32]>,
}

/// Open the world being played to the local network.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenToLan;

/// Stop hosting the world on the local network.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CloseLan;

/// Leave the server, or stop joining it.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LeaveSession;

/// Ask every saved server for its status again and listen for games on the network.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefreshLobby;

/// The network ended the session, or could not start it.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct SessionEnded {
    /// Why, in words for the player.
    pub reason: String,
}

/// Another player joined the session.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PlayerJoined {
    /// The player's name.
    pub name: String,
}

/// Another player left the session.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PlayerLeft {
    /// The player's name.
    pub name: String,
}
