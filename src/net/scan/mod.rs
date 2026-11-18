//! The servers of the multiplayer menu: the status of the saved ones and the games heard on
//! the local network.

pub(in crate::net) mod lan;
pub(in crate::net) mod status;

pub use lan::{Scan, deafen, hear, listen};
pub use status::{Pings, collect, flying, refresh};
