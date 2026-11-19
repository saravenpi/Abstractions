//! State relay: keeping the records players send and copying them to the others each tick.
//!
//! Nothing here allocates once every seat has held a record of each kind: bodies are copied
//! into memory kept from the last record, and batches are built in one reused buffer.

use super::Room;
use super::conn::HELLO_WAIT;
use super::seat::{KINDS, Latest, Sent, kind_index};
use crate::host::{ConnId, DATAGRAM_BUDGET, Outbox};
use crate::ids::StarId;
use crate::wire::{Reason, Record, push_within, records};

/// Seconds between two records of one player and kind sent to a player around another star.
const OTHER_STAR_EVERY: f64 = 1.0;

impl Room {
    /// Keeps the records of a batch from `conn`. A record of another player, a second record
    /// of a kind in the batch, or a record the seat refuses is dropped.
    pub(super) fn ingest(&mut self, conn: ConnId, bytes: &[u8]) {
        let Some(seat) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_mut())
        else {
            return;
        };
        let mut seen = [false; KINDS];
        let player = seat.info.id;
        for record in records(bytes).filter(|record| record.head.player == player) {
            let kind = kind_index(record.head.kind);
            if !seen[kind] {
                seen[kind] = true;
                self.seq += 1;
                seat.keep(&record, self.seq);
            }
        }
    }

    /// Closes every connection that did not join within [`HELLO_WAIT`] seconds.
    pub(super) fn expire(&mut self, now: f64, out: &mut Outbox) {
        self.conns.retain(|conn, open| {
            let keep = open.seat.is_some() || now - open.opened < HELLO_WAIT;
            if !keep {
                out.close(*conn, Reason::Closing);
            }
            keep
        });
    }

    /// Sends every joined player the records of the others they are due.
    pub(super) fn relay(&mut self, now: f64, out: &mut Outbox) {
        for to in 0..self.seats.len() {
            self.relay_to(to, now, out);
        }
    }

    /// Sends the player in seat `to` the newer records of every other seat, in batches of at
    /// most [`DATAGRAM_BUDGET`] bytes.
    fn relay_to(&mut self, to: usize, now: f64, out: &mut Outbox) {
        let Some(me) = &self.seats[to] else {
            return;
        };
        let (conn, star) = (me.conn, me.star);
        self.batch.clear();
        for (from, other) in self.seats.iter().enumerate() {
            let Some(other) = other.as_ref().filter(|_| from != to) else {
                continue;
            };
            for (latest, sent) in other.latest.iter().zip(&mut self.sent[to][from]) {
                let Some(latest) = latest
                    .as_ref()
                    .filter(|latest| due(latest, sent, star, now))
                else {
                    continue;
                };
                offer(&mut self.batch, latest, conn, out);
                *sent = Sent {
                    seq: latest.seq,
                    at: now,
                };
            }
        }
        if !self.batch.is_empty() {
            out.datagram(conn, &self.batch);
        }
    }
}

/// Returns true when `latest` is newer than what was `sent` and, around another star than
/// `star`, the last one went out over a second ago.
fn due(latest: &Latest, sent: &Sent, star: Option<StarId>, now: f64) -> bool {
    let near = star == Some(latest.star);
    latest.seq > sent.seq && (near || now - sent.at >= OTHER_STAR_EVERY)
}

/// Adds `latest` to `batch`. When the batch is full it goes out to `conn` first and the
/// record starts the next one.
fn offer(batch: &mut Vec<u8>, latest: &Latest, conn: ConnId, out: &mut Outbox) {
    let record = Record {
        head: latest.head,
        body: &latest.body,
    };
    if push_within(batch, &record, DATAGRAM_BUDGET) || batch.is_empty() {
        return;
    }
    out.datagram(conn, batch);
    batch.clear();
    push_within(batch, &record, DATAGRAM_BUDGET);
}
