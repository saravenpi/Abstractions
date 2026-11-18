//! What can be done to the saved servers, and the checks on what the player typed.

use crate::settings::SavedServer;

/// Most characters an address may have.
pub const ADDRESS_CHARS: usize = 64;

/// Returns `typed` trimmed, or `refusal` when nothing but spaces was typed.
fn needed(typed: &str, refusal: &str) -> Result<String, String> {
    match typed.trim() {
        "" => Err(refusal.to_string()),
        kept => Ok(kept.to_string()),
    }
}

/// Returns the address to use for a typed one, or why it is refused.
pub fn address(typed: &str) -> Result<String, String> {
    needed(typed, "A server needs an address")
}

/// Returns the player name to store for a typed one, or why it is refused.
pub fn player(typed: &str) -> Result<String, String> {
    needed(typed, "Other players need a name to see")
}

/// Returns the server a form describes: a trimmed name, which may be empty, and an address.
pub fn server(name: &str, address_typed: &str) -> Result<SavedServer, String> {
    Ok(SavedServer {
        name: name.trim().to_string(),
        address: address(address_typed)?,
    })
}

/// Saves `server` over the one at `index`, or at the end without one, and returns its index.
pub fn put(servers: &mut Vec<SavedServer>, index: Option<usize>, server: SavedServer) -> usize {
    match index.filter(|index| *index < servers.len()) {
        Some(index) => {
            servers[index] = server;
            index
        }
        None => {
            servers.push(server);
            servers.len() - 1
        }
    }
}

/// Removes the server at `index`. Returns whether there was one.
pub fn remove(servers: &mut Vec<SavedServer>, index: usize) -> bool {
    let found = index < servers.len();
    if found {
        servers.remove(index);
    }
    found
}
