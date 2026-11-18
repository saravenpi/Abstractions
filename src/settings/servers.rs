//! The servers saved in the multiplayer menu.

use serde::{Deserialize, Serialize};

/// One server the player saved.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(default)]
pub struct SavedServer {
    /// The name the player gave it. Empty shows the address.
    pub name: String,
    /// Where to reach it: a domain or an IP, with an optional port.
    pub address: String,
}

/// Returns the servers a new settings file lists: the public server, which the player can remove.
pub fn default_servers() -> Vec<SavedServer> {
    vec![SavedServer {
        name: "Space".to_string(),
        address: "space.facile.studio".to_string(),
    }]
}
