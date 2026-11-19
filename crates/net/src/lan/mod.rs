//! The LAN beacon: how a game open to LAN announces itself and how others on the network find
//! it.
//!
//! The host broadcasts a small [`Beacon`] once per second on [`crate::wire::BEACON_PORT`]
//! ([`Announcer`]). Others listen on that port and keep what they heard in the last seconds
//! ([`Listener`]). The beacon carries the host's fingerprint, so the player can check the key
//! without ever having seen the server.

mod announce;
mod listen;

#[cfg(test)]
mod beacon_tests;

pub use announce::Announcer;
pub use listen::{Found, Listener};

use serde::{Deserialize, Serialize};

/// First bytes of every beacon: anything else on the port is ignored.
pub const MAGIC: [u8; 4] = *b"ABS1";

/// Most bytes of a beacon, magic included.
pub const BEACON_MAX: usize = 200;

/// What a host says of itself on the local network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beacon {
    /// Protocol the host speaks: compare it with [`crate::wire::PROTOCOL`].
    pub protocol: u16,
    /// Version of the host's game, such as `0.9.0`.
    pub version: String,
    /// Name of the world or the host.
    pub name: String,
    /// Players in the world now.
    pub players: u16,
    /// Most players the host takes.
    pub cap: u16,
    /// UDP port the game listens on.
    pub port: u16,
    /// Fingerprint of the host's certificate.
    pub key: [u8; 32],
}

impl Beacon {
    /// Returns the beacon as one packet, or nothing when it would pass [`BEACON_MAX`] bytes.
    pub fn encode(&self) -> Option<Vec<u8>> {
        let packet = postcard::to_extend(self, MAGIC.to_vec()).ok()?;
        (packet.len() <= BEACON_MAX).then_some(packet)
    }

    /// Reads a beacon from a packet, or nothing when it is not one.
    pub fn decode(packet: &[u8]) -> Option<Self> {
        if packet.len() > BEACON_MAX {
            return None;
        }
        postcard::from_bytes(packet.strip_prefix(&MAGIC)?).ok()
    }
}
