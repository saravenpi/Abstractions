//! The game side of multiplayer: the session state, the server lists, the players and the
//! messages between the interface and the network.
//!
//! This module holds no network code. The menus write the requests and read the state; the
//! network reads the requests, fills the state and writes the answers. Single player never
//! leaves [`Session::Offline`].

mod lobby;
mod messages;
#[cfg(test)]
mod tests;

pub use lobby::{Lobby, Players, RowStatus, ServerRow};
pub use messages::{
    CloseLan, JoinServer, LeaveSession, OpenToLan, PlayerJoined, PlayerLeft, RefreshLobby,
    SessionEnded,
};

use bevy::prelude::*;

/// Where the game stands with other players.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum Session {
    /// Single player: no network at all.
    #[default]
    Offline,
    /// Connecting to a server, before it answered.
    Joining {
        /// The address typed or picked, as the player gave it.
        address: String,
    },
    /// Playing on a server.
    Joined {
        /// The address of the server.
        address: String,
        /// The name the server gives itself.
        name: String,
    },
    /// This game is open to the local network and others can join it.
    Hosting {
        /// The address other players on the network join.
        lan_address: String,
        /// The short key of this host, as `7F3A-91C2-0B5E`.
        key: String,
    },
}

impl Session {
    /// Returns whether other players share the universe: joined to a server, or hosting.
    pub fn shared(&self) -> bool {
        matches!(self, Session::Joined { .. } | Session::Hosting { .. })
    }

    /// Returns whether the game is joining or joined to a server.
    pub fn client(&self) -> bool {
        matches!(self, Session::Joining { .. } | Session::Joined { .. })
    }
}

/// Run condition: joined to a server or hosting. False without the session resource.
pub fn in_session(session: Option<Res<Session>>) -> bool {
    session.is_some_and(|session| session.shared())
}

/// Run condition: single player. True without the session resource.
pub fn offline(session: Option<Res<Session>>) -> bool {
    session.is_none_or(|session| *session == Session::Offline)
}

/// Registers the session state, the lists and every message of the interface.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Session>()
        .init_resource::<Lobby>()
        .init_resource::<Players>()
        .add_message::<JoinServer>()
        .add_message::<OpenToLan>()
        .add_message::<CloseLan>()
        .add_message::<LeaveSession>()
        .add_message::<RefreshLobby>()
        .add_message::<SessionEnded>()
        .add_message::<PlayerJoined>()
        .add_message::<PlayerLeft>();
}
