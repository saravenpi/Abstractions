//! Requests the interface sends to the network, and what the network tells the interface.

use bevy::prelude::*;

/// Join the server at `address`.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct JoinServer {
    /// The address typed or picked.
    pub address: String,
    /// The fingerprint the server must show, when a LAN beacon announced it.
    pub expected_key: Option<[u8; 32]>,
    /// The password of the server, when the player typed one.
    pub password: Option<String>,
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

/// A line for the banner from the network, such as the key of a server seen the first time.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct SessionNotice {
    /// The line, in words for the player.
    pub line: String,
}

/// The server refused the password: the interface asks for one and joins again.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct PasswordNeeded {
    /// The address that was joined.
    pub address: String,
    /// The fingerprint the server had to show, when a LAN beacon announced it.
    pub expected_key: Option<[u8; 32]>,
}
