//! Opening a single-player world to the local network: the same room the dedicated server
//! runs, on a thread of this game, a beacon so others find it, and this game joined to its own
//! server as a normal player without restarting the world.
//!
//! The host's world file stays the single source of truth: when hosting stops, the room's log
//! is merged back and the world is saved as in single player.

mod beacon;
mod close;
pub(in crate::net) mod open;

pub use beacon::count;
pub use close::{Resave, close_lan, close_on_exit, close_on_leave, resave};
pub use open::open_lan;

use std::time::Instant;

use abstractions_net::io::ServerHandle;
use abstractions_net::lan::{Announcer, Beacon};
use abstractions_net::room::Room;
use bevy::prelude::*;

/// A world open to the local network: its server, its beacon and when the room started.
#[derive(Resource, Debug)]
pub struct Hosted {
    /// The server, until hosting stops.
    server: Option<ServerHandle<Room>>,
    /// The beacon thread, when the network allows broadcasts.
    announcer: Option<Announcer>,
    /// What the beacon says now.
    beacon: Beacon,
    /// When the room started: its network time counts from here.
    started: Instant,
}
