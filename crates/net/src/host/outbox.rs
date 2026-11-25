//! The queue of what the host wants sent, reused from tick to tick.

use std::ops::Range;

use super::ConnId;
use crate::wire::{Reason, ServerMsg};

/// One thing the host asks the network to do.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Send a message on the connection's reliable stream.
    Send(ConnId, ServerMsg),
    /// Send a datagram whose bytes sit in [`Outbox::bytes`] at this range.
    Datagram(ConnId, Range<usize>),
    /// Close the connection, telling the player why.
    Close(ConnId, Reason),
}

/// What the host wants sent, kept between ticks so its buffers are reused.
#[derive(Debug, Default)]
pub struct Outbox {
    /// Every operation, in the order asked.
    ops: Vec<Op>,
    /// The bytes of every datagram, back to back.
    bytes: Vec<u8>,
}

impl Outbox {
    /// Queues a message on the reliable stream of `conn`.
    pub fn send(&mut self, conn: ConnId, msg: ServerMsg) {
        self.ops.push(Op::Send(conn, msg));
    }

    /// Queues a datagram of `bytes` to `conn`. The bytes are copied once.
    pub fn datagram(&mut self, conn: ConnId, bytes: &[u8]) {
        let start = self.bytes.len();
        self.bytes.extend_from_slice(bytes);
        self.ops.push(Op::Datagram(conn, start..self.bytes.len()));
    }

    /// Queues the closing of `conn` with `reason`.
    pub fn close(&mut self, conn: ConnId, reason: Reason) {
        self.ops.push(Op::Close(conn, reason));
    }

    /// Returns the queued operations, oldest first.
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// Returns the bytes of a datagram named by an [`Op::Datagram`] range.
    pub fn bytes(&self, range: Range<usize>) -> &[u8] {
        &self.bytes[range]
    }

    /// Forgets every queued operation and keeps the memory for the next tick.
    pub fn clear(&mut self) {
        self.ops.clear();
        self.bytes.clear();
    }
}
