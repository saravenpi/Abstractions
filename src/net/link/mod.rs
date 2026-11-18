//! The link to a server: the one way the game talks to the network.
//!
//! [`Link`] exists only while the game joins, plays on or hosts a server. Every message and
//! datagram goes through it, and [`drain`] reads what arrived once per frame, at most
//! [`EVENTS_PER_FRAME`] events, so a burst never stalls a frame.
//!
//! The seam for the state of remote players: datagram batches that arrived this frame wait in
//! [`Inbox`] for systems that run after [`Receive`]; [`Link::send_records`] sends one batch,
//! which must not pass [`Link::max_datagram`]; [`Link::you`] is the id the server gave this
//! player, whose own records come back and must be skipped.

mod drain;
mod source;

pub use drain::{Dropped, EVENTS_PER_FRAME, FreshKey, FromServer, Inbox, Receive, drain};
pub use source::Source;
#[cfg(test)]
pub use source::{Script, Wire};

use std::time::Instant;

use abstractions_net::io::{Client, ConnectOpts};
use abstractions_net::log::PlayerId;
use abstractions_net::wire::ClientMsg;
use bevy::prelude::*;

/// What the link is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    /// Joining a server: its world replaces the one on screen.
    Join {
        /// The address as the player gave it.
        address: String,
        /// The fingerprint the server must show, when a LAN beacon announced it.
        expected: Option<[u8; 32]>,
    },
    /// This game hosts the world and joins its own server: the world on screen stays.
    Host,
}

/// The connection of this game to a server, with what the session learned of it.
#[derive(Resource, Debug)]
pub struct Link {
    /// The other end, until the link is closed.
    pub(in crate::net) source: Option<Source>,
    /// What the link is for.
    pub role: Role,
    /// The id the server gave this player, once welcomed.
    pub you: Option<PlayerId>,
    /// The start of the local monotonic clock pings are measured on.
    origin: Instant,
}

impl Link {
    /// Starts connecting to `address` on a thread of its own and returns at once.
    pub fn connect(address: &str, opts: ConnectOpts, role: Role) -> Self {
        Self::over(Source::Live(Client::connect(address, opts)), role)
    }

    /// Returns a link over `source`.
    pub fn over(source: Source, role: Role) -> Self {
        Self {
            source: Some(source),
            role,
            you: None,
            origin: Instant::now(),
        }
    }

    /// Returns the local monotonic time in seconds, the clock pings carry.
    pub fn now(&self) -> f64 {
        self.origin.elapsed().as_secs_f64()
    }

    /// Queues a reliable message for the server.
    pub fn send(&mut self, msg: ClientMsg) {
        if let Some(source) = &mut self.source {
            source.send(msg);
        }
    }

    /// Sends one batch of state records in a datagram. A batch longer than
    /// [`Self::max_datagram`] is dropped, so split batches with that limit.
    pub fn send_records(&mut self, batch: &[u8]) {
        if let Some(source) = &mut self.source {
            source.datagram(batch);
        }
    }

    /// Returns the most bytes a datagram may carry now, 0 before the connection is up.
    pub fn max_datagram(&self) -> usize {
        self.source.as_ref().map_or(0, Source::max_datagram)
    }

    /// Says goodbye and closes the connection. With `wait` it returns once it is over, which
    /// is for the last frame of the game only.
    pub fn close(&mut self, wait: bool) {
        if let Some(mut source) = self.source.take() {
            source.send(ClientMsg::Bye);
            source.close(wait);
        }
    }
}
