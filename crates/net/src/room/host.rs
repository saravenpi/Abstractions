//! The room as the network calls it.

use std::net::SocketAddr;

use super::Room;
use super::conn::{CONNS_PER_IP, Conn, MESSAGES_PER_SECOND};
use crate::host::{ConnId, Host, Outbox};
use crate::wire::{ClientMsg, Reason, ServerMsg};

impl Host for Room {
    /// Takes the connection, unless its IP address already holds the most connections.
    fn connected(&mut self, conn: ConnId, addr: SocketAddr, now: f64, out: &mut Outbox) {
        let ip = addr.ip();
        if self.conns.values().filter(|open| open.ip == ip).count() >= CONNS_PER_IP {
            out.close(conn, Reason::Full);
            return;
        }
        self.conns.insert(conn, Conn::new(ip, now));
    }

    /// Answers one message, and closes a connection that sends too many.
    fn message(&mut self, conn: ConnId, msg: ClientMsg, now: f64, out: &mut Outbox) {
        let Some(open) = self.conns.get_mut(&conn) else {
            return;
        };
        open.heard = now;
        if !open.messages.allow(now, MESSAGES_PER_SECOND) {
            self.forget(conn, out);
            out.close(conn, Reason::Flooding);
            return;
        }
        match msg {
            ClientMsg::Status => self.status(conn, out),
            ClientMsg::Hello(hello) => self.hello(conn, hello, now, out),
            ClientMsg::Change(change) => self.change(conn, change, now, out),
            ClientMsg::Record(record) => self.record(conn, record),
            ClientMsg::Ping { id, sent } => {
                let clock = self.clock(now);
                out.send(conn, ServerMsg::Pong { id, sent, clock });
            }
            ClientMsg::Bye => self.leave(conn, out),
            ClientMsg::Explored { body, runs } => self.explored(conn, body, &runs),
            ClientMsg::Waypoint(waypoint) => self.waypoint(conn, waypoint, out),
            ClientMsg::Hunt(act) => self.hunt(conn, act, now, out),
        }
    }

    /// Keeps the state records of a batch, and notes that the connection was heard.
    fn datagram(&mut self, conn: ConnId, bytes: &[u8], now: f64, _out: &mut Outbox) {
        if let Some(open) = self.conns.get_mut(&conn) {
            open.heard = now;
        }
        self.ingest(conn, bytes);
    }

    /// Lets the player go, if the connection had joined.
    fn closed(&mut self, conn: ConnId, _now: f64, out: &mut Outbox) {
        self.forget(conn, out);
    }

    /// Closes connections that never joined and frees the seats of players gone silent, sends
    /// every player the others' state, then the newly explored ground that is due.
    fn tick(&mut self, now: f64, out: &mut Outbox) {
        self.expire(now, out);
        self.relay(now, out);
        self.spread_explored(now, out);
        self.sweep(now);
    }

    /// Returns true while any connection is open.
    fn busy(&self) -> bool {
        !self.conns.is_empty()
    }
}
