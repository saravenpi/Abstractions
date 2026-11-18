//! The end of a session: a refusal, a kick or a closed connection, said in plain words.

use abstractions_net::io::CloseReason;
use abstractions_net::trust::TrustError;
use abstractions_net::wire::{Reason, ServerMsg};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::shut;
use crate::net::link::{Dropped, FromServer, Link, Role};
use crate::net::{CloseLan, Lobby, PasswordNeeded, Players, RowStatus, Session, SessionEnded};

/// Returns why a session ended in words for the player. `version` is the version the server
/// runs, when its status is known.
pub fn words(reason: &CloseReason, address: &str, version: Option<&str>) -> String {
    match reason {
        CloseReason::Refused(reason) => refusal(*reason, version),
        CloseReason::Trust(TrustError::Changed { .. }) => "This server's key changed".to_string(),
        CloseReason::Trust(error) => error.to_string(),
        CloseReason::Address(error) => error.to_string(),
        CloseReason::Timeout | CloseReason::Unreachable => format!("No answer from {address}"),
        CloseReason::Lost => "Connection lost".to_string(),
    }
}

/// Returns why the server refused or sent the player away, in words.
fn refusal(reason: Reason, version: Option<&str>) -> String {
    let named =
        |fallback: &str| version.map_or(fallback.to_string(), |version| format!("v{version}"));
    match reason {
        Reason::WrongPassword => "Wrong password".to_string(),
        Reason::Full => "The server is full".to_string(),
        Reason::Outdated => format!("This server needs {}", named("a newer game")),
        Reason::Newer => format!("You need {}", named("an older game")),
        Reason::Flooding => "The server sent you away for sending too much".to_string(),
        Reason::Closing => "The server closed".to_string(),
    }
}

/// What the end of a session changes and says.
#[derive(SystemParam)]
pub struct Ending<'w, 's> {
    /// Removes the link.
    commands: Commands<'w, 's>,
    /// Where the game stands with other players.
    session: ResMut<'w, Session>,
    /// The players of the session, forgotten.
    players: ResMut<'w, Players>,
    /// The version of the server, for an outdated game.
    lobby: Res<'w, Lobby>,
    /// Why the session ended, for the interface.
    ended: MessageWriter<'w, SessionEnded>,
    /// The request for a password after a wrong one.
    password: MessageWriter<'w, PasswordNeeded>,
    /// The end of hosting, when the link of a host breaks.
    close_lan: MessageWriter<'w, CloseLan>,
}

impl Ending<'_, '_> {
    /// Forgets the session of `role`, which ended for `reason`, and tells the interface. A
    /// wrong password asks for one. A host whose own link broke stops hosting.
    fn finish(&mut self, role: &Role, reason: &CloseReason) {
        shut(&mut self.commands);
        self.players.0.clear();
        let Role::Join { address, expected } = role else {
            self.close_lan.write(CloseLan);
            return;
        };
        *self.session = Session::Offline;
        let version = match self.lobby.find(address).map(|row| &row.status) {
            Some(RowStatus::Online { version, .. }) => Some(version.as_str()),
            _ => None,
        };
        let reason_text = words(reason, address, version);
        self.ended.write(SessionEnded {
            reason: reason_text,
        });
        if *reason == CloseReason::Refused(Reason::WrongPassword) {
            self.password.write(PasswordNeeded {
                address: address.clone(),
                expected_key: *expected,
            });
        }
    }
}

/// Ends the session on a refusal, a kick or a closed connection, once.
pub fn end(
    mut from: MessageReader<FromServer>,
    mut dropped: MessageReader<Dropped>,
    mut link: ResMut<Link>,
    mut ending: Ending,
) {
    let refused = from.read().find_map(|FromServer(msg)| match msg {
        ServerMsg::Refused(reason) | ServerMsg::Kick(reason) => Some(CloseReason::Refused(*reason)),
        _ => None,
    });
    let closed = dropped.read().last().map(|Dropped(reason)| reason.clone());
    let Some(reason) = refused.or(closed) else {
        return;
    };
    link.close(false);
    ending.finish(&link.role, &reason);
}
