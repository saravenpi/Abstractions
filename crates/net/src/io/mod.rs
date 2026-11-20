//! The network itself: QUIC endpoints for the server and for a player.
//!
//! [`serve`] runs a server on its own thread and drives a [`crate::host::Host`]. [`Client`]
//! connects a player from its own thread and hands events to the game loop without blocking.
//! [`status`] asks a server what it is, for the server list. Both sides speak QUIC through
//! quinn on smol: datagrams carry state, one bidirectional stream carries messages.

pub mod addr;
mod client;
mod close;
mod config;
mod link;
mod server;
mod words;

#[cfg(test)]
mod addr_tests;
#[cfg(test)]
mod close_tests;
#[cfg(test)]
mod words_tests;

pub use addr::{AddrError, resolve};
pub use client::{Client, ClientEvent, CloseReason, ConnectOpts, status};
pub use server::{ServeConfig, ServerHandle, serve};
