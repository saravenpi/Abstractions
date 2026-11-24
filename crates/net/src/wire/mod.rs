//! The protocol: what travels between a player and a server, byte for byte.
//!
//! State (where a robot, a ship or a companion is) travels in unreliable datagrams as batches
//! of [`Record`]s. Everything else travels in reliable streams as [`ClientMsg`] and
//! [`ServerMsg`], each behind a length ([`encode`], [`decode`]). The server reads the head of
//! a record and never its body.

mod act;
mod body;
mod bytes;
mod cadence;
mod companion;
mod field;
mod fleet;
mod frame;
mod ground;
mod hunt;
mod msg;
mod name;
mod record;
mod robot;
mod ship;
mod stream;
mod varint;
mod vitals;

#[cfg(test)]
mod body_tests;
#[cfg(test)]
mod cadence_tests;
#[cfg(test)]
mod cells_tests;
#[cfg(test)]
mod far_tests;
#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod ground_tests;
#[cfg(test)]
mod hunt_tests;
#[cfg(test)]
mod kind_tests;
#[cfg(test)]
mod msg_tests;
#[cfg(test)]
mod name_tests;
#[cfg(test)]
mod order_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod stream_tests;
#[cfg(test)]
mod switch_tests;

pub use act::{Act, Event, HuntWorld};
pub use body::{BODY_MAX, Body};
pub use cadence::{Cadence, Cadences, MOVING, RESTING};
pub use companion::{CompanionBody, LIFT_RANGE};
pub use field::{
    Attack, Blow, CLEAN, DEFLECTED, Impact, MISS, Opened, Rock, STOPPED, Wound, finite,
};
pub use fleet::{ShipAct, ShipEvent, ShipPilot};
pub use frame::{Frame, Kind};
pub use ground::{GROUND_CHUNK, HEAP, LYING_BYTES, LYING_MASS, Lying, TOOL};
pub use hunt::{
    AnimalKey, CARCASSES_MAX, CREDIT_WINDOW, Carcass, Hit, ITEM_MAX, Kill, LASTS, PARTS_MAX, SKEW,
    SPEARS_MAX, Spear, SpearKey,
};
pub use msg::{
    ClientMsg, Hello, PlayerInfo, PlayerRecord, Reason, ServerMsg, Status, Waypoint, Welcome,
};
pub use name::clean_name;
pub use record::{AT_REST, Head, Record, Records, push, push_within, records, split};
pub use robot::{
    AIRSPEED_RANGE, CLIMB_RANGE, PHASE_RANGE, RobotBody, SPEED_RANGE, STRIDE_WRAP, stride_delta,
};
pub use ship::{ShipBody, WARP_ACTIVE, WARP_OFF, WARP_SPOOLING};
pub use stream::{CLIENT_MESSAGE_MAX, MESSAGE_MAX, StreamError, decode, encode};
pub use vitals::{HARM_PARTS, VitalsBody};

/// Version of the protocol. A player and a server must hold the same number to play together.
/// Raise it whenever bytes on the wire change meaning.
pub const PROTOCOL: u16 = 5;

/// Default UDP port of the game.
pub const GAME_PORT: u16 = 24650;

/// UDP port a host announces itself on, on the local network.
pub const BEACON_PORT: u16 = 24651;

/// Most players a server takes unless its owner says otherwise. It is a default, never a
/// ceiling.
pub const MAX_PLAYERS_DEFAULT: u16 = 16;

/// Most players an owner may let into a server.
pub const MAX_PLAYERS_LIMIT: u16 = 1000;

/// Most characters in a player name.
pub const NAME_MAX: usize = 24;
