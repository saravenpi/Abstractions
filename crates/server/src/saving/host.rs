//! The saving room as the network calls it: every call goes to the room.

use std::net::SocketAddr;

use abstractions_net::host::{ConnId, Host, Op, Outbox};
use abstractions_net::wire::{ClientMsg, Reason};

use super::Saving;

impl Host for Saving {
    /// Hands the new connection to the room.
    fn connected(&mut self, conn: ConnId, addr: SocketAddr, now: f64, out: &mut Outbox) {
        self.room.connected(conn, addr, now, out);
    }

    /// Hands the message to the room. Only a hello, a goodbye or a close can change who is
    /// in, so only then are the players looked at. A hello that took the seat of an older
    /// session of the same player is logged: the players themselves did not change.
    fn message(&mut self, conn: ConnId, msg: ClientMsg, now: f64, out: &mut Outbox) {
        let roster = matches!(msg, ClientMsg::Hello(_) | ClientMsg::Bye);
        self.room.message(conn, msg, now, out);
        if replaces(out) {
            println!("A player joined again: the older session was closed");
        }
        if roster || closes(out, conn) {
            self.check(now);
        }
    }

    /// Hands the datagram to the room.
    fn datagram(&mut self, conn: ConnId, bytes: &[u8], now: f64, out: &mut Outbox) {
        self.room.datagram(conn, bytes, now, out);
    }

    /// Hands the close to the room and logs the player who left.
    fn closed(&mut self, conn: ConnId, now: f64, out: &mut Outbox) {
        self.room.closed(conn, now, out);
        self.check(now);
    }

    /// Ticks the room, logs the players it let go for their silence, and saves when a save is
    /// due.
    fn tick(&mut self, now: f64, out: &mut Outbox) {
        self.room.tick(now, out);
        if out.ops().iter().any(|op| matches!(op, Op::Close(..))) {
            self.check(now);
        }
        self.due(now);
    }

    /// Returns true while the room is busy or holds something not saved yet, so a save that
    /// the saver thread had no room for is tried again even after the last player left.
    fn busy(&self) -> bool {
        self.room.busy() || self.room.changed()
    }
}

/// Returns true when `out` holds the close of a session replaced by a newer one.
fn replaces(out: &Outbox) -> bool {
    let replaced = |op: &Op| matches!(op, Op::Close(_, Reason::Replaced));
    out.ops().iter().any(replaced)
}

/// Returns true when `out` holds a close of `conn`.
fn closes(out: &Outbox, conn: ConnId) -> bool {
    out.ops()
        .iter()
        .any(|op| matches!(op, Op::Close(id, _) if *id == conn))
}
