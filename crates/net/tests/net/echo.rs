//! A tiny host for the tests: it answers status, welcomes players, answers pings, refuses the
//! name `full` and sends every datagram back.

use std::collections::BTreeSet;
use std::net::SocketAddr;

use abstractions_net::host::{ConnId, Host, Outbox};
use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{ClientMsg, PROTOCOL, Reason, ServerMsg, Status, Welcome};

/// The test host: the connections it knows and how many it ever saw.
#[derive(Debug, Default)]
pub struct Echo {
    /// Connections open now.
    pub open: BTreeSet<ConnId>,
    /// Connections ever made.
    pub seen: usize,
}

impl Host for Echo {
    /// Counts the connection.
    fn connected(&mut self, conn: ConnId, _addr: SocketAddr, _now: f64, _out: &mut Outbox) {
        self.open.insert(conn);
        self.seen += 1;
    }

    /// Answers status, hello and ping.
    fn message(&mut self, conn: ConnId, msg: ClientMsg, now: f64, out: &mut Outbox) {
        match msg {
            ClientMsg::Status => out.send(conn, ServerMsg::Status(status(self.open.len()))),
            ClientMsg::Hello(hello) if hello.name == "full" => {
                out.send(conn, ServerMsg::Refused(Reason::Full));
                out.close(conn, Reason::Full);
            }
            ClientMsg::Hello(_) => out.send(conn, ServerMsg::Welcome(welcome(conn, now))),
            ClientMsg::Ping { id, sent } => out.send(
                conn,
                ServerMsg::Pong {
                    id,
                    sent,
                    clock: now,
                },
            ),
            _ => {}
        }
    }

    /// Sends the datagram back.
    fn datagram(&mut self, conn: ConnId, bytes: &[u8], _now: f64, out: &mut Outbox) {
        out.datagram(conn, bytes);
    }

    /// Forgets the connection.
    fn closed(&mut self, conn: ConnId, _now: f64, _out: &mut Outbox) {
        self.open.remove(&conn);
    }

    /// Does nothing: the test host has no state to send.
    fn tick(&mut self, _now: f64, _out: &mut Outbox) {}

    /// Wants ticks while anyone is connected.
    fn busy(&self) -> bool {
        !self.open.is_empty()
    }
}

/// Returns the status of the test server with `players` connected.
fn status(players: usize) -> Status {
    Status {
        name: "test".to_string(),
        players: players as u16,
        cap: 16,
        version: "0.9.0".to_string(),
        protocol: PROTOCOL,
    }
}

/// Returns the welcome of a player.
fn welcome(conn: ConnId, now: f64) -> Welcome {
    Welcome {
        you: PlayerId(conn.0 as u32),
        seed: 7,
        home: abstractions_net::store::HOME_RULE,
        clock: now,
        changes: Changes::default(),
        atlas: String::new(),
        record: None,
        players: Vec::new(),
        explored: Vec::new(),
        waypoints: Vec::new(),
    }
}
