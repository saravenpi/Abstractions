//! The protocol: what travels between a player and a server, byte for byte.
//!
//! State (where a robot, a ship or a companion is) travels in unreliable datagrams as batches
//! of [`Record`]s. Everything else travels in reliable streams as [`ClientMsg`] and
//! [`ServerMsg`], each behind a length ([`encode`], [`decode`]). The server reads the head of
//! a record and never its body.

mod bytes;
mod companion;
mod frame;
mod msg;
mod name;
mod record;
mod robot;
mod ship;
mod stream;

#[cfg(test)]
mod body_tests;
#[cfg(test)]
mod msg_tests;
#[cfg(test)]
mod name_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod stream_tests;

pub use companion::{CompanionBody, LIFT_RANGE};
pub use frame::{Frame, Kind};
pub use msg::{
    ClientMsg, Hello, PlayerInfo, PlayerRecord, Reason, ServerMsg, Status, Waypoint, Welcome,
};
pub use name::clean_name;
pub use record::{AT_REST, Head, Record, Records, push, push_within, records, split};
pub use robot::{AIRSPEED_RANGE, CLIMB_RANGE, RobotBody, SPEED_RANGE, STRIDE_WRAP, stride_delta};
pub use ship::{ShipBody, WARP_ACTIVE, WARP_OFF, WARP_SPOOLING};
pub use stream::{CLIENT_MESSAGE_MAX, MESSAGE_MAX, StreamError, decode, encode};

/// Version of the protocol. A player and a server must hold the same number to play together.
/// Raise it whenever bytes on the wire change meaning.
pub const PROTOCOL: u16 = 2;

/// Columns of the explored grid of a body, longitude 0 first, going east.
pub const EXPLORED_COLUMNS: u32 = 512;

/// Rows of the explored grid of a body, the south pole first.
pub const EXPLORED_ROWS: u32 = 256;

/// Default UDP port of the game.
pub const GAME_PORT: u16 = 24650;

/// UDP port a host announces itself on, on the local network.
pub const BEACON_PORT: u16 = 24651;

/// Most players a server takes unless its owner says otherwise.
pub const MAX_PLAYERS_DEFAULT: u8 = 16;

/// Most characters in a player name.
pub const NAME_MAX: usize = 24;
