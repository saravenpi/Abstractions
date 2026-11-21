//! Why a connection ended, in plain words, said once for every reader: the screen of a player,
//! the `ping` of a server, and the text a close carries for the logs of the other side.

use super::client::CloseReason;
use crate::trust::TrustError;
use crate::wire::Reason;

impl Reason {
    /// Returns why the server refused or sent a player away, in words. `version` is the version
    /// the server runs, when it is known: it names what an outdated side needs.
    pub fn words(self, version: Option<&str>) -> String {
        let named =
            |fallback: &str| version.map_or(fallback.to_string(), |version| format!("v{version}"));
        match self {
            Self::WrongPassword => "Wrong password".to_string(),
            Self::Full => "The server is full".to_string(),
            Self::Outdated => format!("This server needs {}", named("a newer game")),
            Self::Newer => format!("You need {}", named("an older game")),
            Self::Flooding => "The server sent you away for sending too much".to_string(),
            Self::Closing => "The server closed".to_string(),
            Self::Replaced => "You joined this server from another game".to_string(),
            Self::Silent => "The server stopped hearing from your game".to_string(),
        }
    }
}

impl CloseReason {
    /// Returns why a connection to `address` ended, in words for the player. `version` is the
    /// version the server runs, when its status is known.
    pub fn words(&self, address: &str, version: Option<&str>) -> String {
        match self {
            Self::Refused(reason) => reason.words(version),
            Self::Trust(TrustError::Changed { .. }) => "This server's key changed".to_string(),
            Self::Trust(error) => error.to_string(),
            Self::Address(error) => error.to_string(),
            Self::Timeout | Self::Unreachable => format!("No answer from {address}"),
            Self::Lost => "Connection lost".to_string(),
        }
    }
}
