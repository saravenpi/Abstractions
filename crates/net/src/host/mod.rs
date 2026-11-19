//! The seam between the network and the server's logic.
//!
//! [`crate::io`] owns the sockets and calls a [`Host`] for every connection, message and
//! datagram, and once per tick. The host answers by filling an [`Outbox`], which the network
//! then sends. [`crate::room::Room`] is the host of every server, dedicated or open to LAN.
//! Time is a monotonic count of seconds since the network started, never the wall clock.

mod outbox;

pub use outbox::{Op, Outbox};

use std::net::SocketAddr;

use crate::wire::ClientMsg;

/// One connection to the server, unique for the life of the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnId(pub u64);

/// The server's logic as the network sees it. Every call may queue operations in `out`.
pub trait Host {
    /// A player's connection finished its handshake from `addr`.
    fn connected(&mut self, conn: ConnId, addr: SocketAddr, now: f64, out: &mut Outbox);

    /// A whole message arrived on the reliable stream of `conn`.
    fn message(&mut self, conn: ConnId, msg: ClientMsg, now: f64, out: &mut Outbox);

    /// A datagram arrived from `conn`.
    fn datagram(&mut self, conn: ConnId, bytes: &[u8], now: f64, out: &mut Outbox);

    /// The connection is gone, closed by either side or timed out.
    fn closed(&mut self, conn: ConnId, now: f64, out: &mut Outbox);

    /// One step of the loop, at [`TICK_HZ`] while anyone is connected.
    fn tick(&mut self, now: f64, out: &mut Outbox);

    /// Returns true while the host wants ticks, so an empty server can sleep.
    fn busy(&self) -> bool;
}

/// Ticks per second of the server loop.
pub const TICK_HZ: f64 = 30.0;

/// Most bytes one datagram batch may hold. Quinn measured 1162 on loopback; this leaves room
/// for paths with a smaller limit.
pub const DATAGRAM_BUDGET: usize = 1100;
