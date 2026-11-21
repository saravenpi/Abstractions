//! State relay: keeping the records players send and copying them to the others each tick.
//!
//! Each seat keeps the last [`super::seat::DEPTH`] records of each kind, and a player around
//! the same star gets every one sent since the last tick, in order. Nothing here allocates
//! once every seat has filled its slots: bodies are copied into memory kept from older
//! records, and batches are built in one reused buffer.

use super::Room;
use super::conn::{Conn, HELLO_WAIT, SILENT_WAIT};
use super::seat::{Latest, Recent, Sent};
use crate::host::{ConnId, DATAGRAM_BUDGET, Outbox};
use crate::ids::StarId;
use crate::wire::{Reason, Record, push_within, records};

/// Seconds between two records of one player and kind sent to a player around another star.
const OTHER_STAR_EVERY: f64 = 1.0;

impl Room {
    /// Keeps the records of a batch from `conn`, in order. A record of another player or a
    /// record the seat refuses is dropped.
    pub(super) fn ingest(&mut self, conn: ConnId, bytes: &[u8]) {
        let Some(seat) = self
            .seat_of(conn)
            .and_then(|slot| self.seats[slot].as_mut())
        else {
            return;
        };
        let player = seat.info.id;
        for record in records(bytes).filter(|record| record.head.player == player) {
            self.seq += 1;
            seat.keep(&record, self.seq);
        }
    }

    /// Closes every connection that did not join within [`HELLO_WAIT`] seconds, and every joined
    /// one not heard for [`SILENT_WAIT`] seconds: its player leaves, and the record they sent
    /// last stays. Nothing is allocated while no connection is late.
    pub(super) fn expire(&mut self, now: f64, out: &mut Outbox) {
        let late = |open: &Conn| match open.seat {
            Some(_) => (now - open.heard >= SILENT_WAIT).then_some(Reason::Silent),
            None => (now - open.opened >= HELLO_WAIT).then_some(Reason::Closing),
        };
        let found = self.conns.iter();
        let gone: Vec<(ConnId, Reason)> = found
            .filter_map(|(conn, open)| Some((*conn, late(open)?)))
            .collect();
        for (conn, reason) in gone {
            self.forget(conn, out);
            out.close(conn, reason);
        }
    }

    /// Sends every joined player the records of the others they are due.
    pub(super) fn relay(&mut self, now: f64, out: &mut Outbox) {
        for to in 0..self.seats.len() {
            self.relay_to(to, now, out);
        }
        self.relayed = self.seq;
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
            for (recent, sent) in other.recent.iter().zip(&mut self.sent[to][from]) {
                let Some(newest) = recent
                    .newest()
                    .filter(|newest| due(newest, sent, star, now))
                else {
                    continue;
                };
                let near = star == Some(newest.star);
                let since = sent.seq.max(self.relayed);
                for latest in owed(recent, newest, since, near) {
                    offer(&mut self.batch, latest, conn, out);
                }
                *sent = Sent {
                    seq: newest.seq,
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

/// Returns the records of `recent` a recipient is owed: around the same star every one above
/// number `since`, oldest first, else only `newest`.
fn owed<'a>(
    recent: &'a Recent,
    newest: &'a Latest,
    since: u64,
    near: bool,
) -> impl Iterator<Item = &'a Latest> {
    let older = recent
        .iter()
        .filter(move |latest| near && latest.seq > since);
    older.chain((!near).then_some(newest))
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
