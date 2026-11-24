//! Batched surface creature state relay: keeps recent snapshots per sender without steady-state
//! allocations and forwards same-star updates to other players.

use super::Room;
use super::relay::offer;
use super::seat::Latest;
use crate::host::{ConnId, Outbox};
use crate::ids::StarId;
use crate::log::PlayerId;
use crate::wire::{FAUNA_BODY_MAX, Frame, Head, Record};

/// Most creature batch records kept per player seat between room ticks.
const FAUNA_DEPTH: usize = 4;

/// Circular buffer of recent [`crate::wire::Kind::Fauna`] records, reusing body memory once warm.
#[derive(Debug, Default)]
pub(super) struct FaunaRecent {
    /// Kept records, with `seq == 0` marking cleared slots whose memory stays allocated.
    slots: Vec<Latest>,
    /// Next slot index to overwrite once [`FAUNA_DEPTH`] buffers are allocated.
    start: usize,
}

impl FaunaRecent {
    /// Returns active records from oldest to newest.
    pub(super) fn iter(&self) -> impl Iterator<Item = &Latest> {
        let (newer, older) = self.slots.split_at(self.start);
        older.iter().chain(newer).filter(|slot| slot.seq > 0)
    }

    /// Returns the active record with the highest sequence number, if any.
    pub(super) fn newest(&self) -> Option<&Latest> {
        self.slots
            .iter()
            .filter(|slot| slot.seq > 0)
            .max_by_key(|slot| slot.seq)
    }

    /// Marks all slots empty while retaining allocated slot and body buffers.
    pub(super) fn clear(&mut self) {
        for slot in &mut self.slots {
            slot.seq = 0;
            slot.head.time = f64::NEG_INFINITY;
            slot.body.clear();
        }
        self.start = 0;
    }

    /// Keeps `record` under `seq`, allowing equal timestamps for multi-chunk batches of one frame.
    pub(super) fn keep(&mut self, record: &Record<'_>, seq: u64) -> Option<StarId> {
        let older = self
            .newest()
            .is_some_and(|newest| record.head.time < newest.head.time);
        if record.body.len() > FAUNA_BODY_MAX || !record.head.time.is_finite() || older {
            return None;
        }
        let star = record.head.frame.star();
        let latest = if let Some(idx) = self.slots.iter().position(|slot| slot.seq == 0) {
            if self.slots.len() == FAUNA_DEPTH {
                self.start = (idx + 1) % FAUNA_DEPTH;
            }
            &mut self.slots[idx]
        } else if self.slots.len() < FAUNA_DEPTH {
            self.slots.push(Latest {
                head: Head { ..record.head },
                body: Vec::with_capacity(FAUNA_BODY_MAX),
                star,
                seq: 0,
            });
            let last = self.slots.len() - 1;
            &mut self.slots[last]
        } else {
            let slot = self.start;
            self.start = (self.start + 1) % FAUNA_DEPTH;
            &mut self.slots[slot]
        };
        latest.head = record.head;
        latest.body.clear();
        latest.body.extend_from_slice(record.body);
        latest.star = star;
        latest.seq = seq;
        Some(star)
    }
}

/// Pre-allocated creature state relay storage for every player seat in the room.
#[derive(Debug)]
pub(super) struct FaunaRelay {
    /// Recent creature batches per sender seat index.
    pub(super) recent: Vec<FaunaRecent>,
    /// Last relayed sequence watermark: `sent[to][from]`.
    pub(super) sent: Vec<Vec<u64>>,
}

impl FaunaRelay {
    /// Allocates empty creature buffers and watermarks for `cap` seats.
    pub(super) fn new(cap: usize) -> Self {
        Self {
            recent: (0..cap).map(|_| FaunaRecent::default()).collect(),
            sent: vec![vec![0; cap]; cap],
        }
    }

    /// Clears creature buffers and send watermarks for seat `slot` when a player joins.
    pub(super) fn reset_slot(&mut self, slot: usize) {
        self.recent[slot].clear();
        self.sent[slot].fill(0);
        for row in &mut self.sent {
            row[slot] = 0;
        }
    }
}

impl Room {
    /// Stores an incoming [`crate::wire::Kind::Fauna`] record from `player` on a planet surface.
    pub(super) fn ingest_fauna(&mut self, slot: usize, player: PlayerId, record: &Record<'_>) {
        if record.head.player != player || !matches!(record.head.frame, Frame::Surface(_)) {
            return;
        }
        let next_seq = self.seq + 1;
        let Some(star) = self.fauna.recent[slot].keep(record, next_seq) else {
            return;
        };
        self.seq = next_seq;
        if let Some(seat) = self.seats[slot].as_mut() {
            seat.star = Some(star);
        }
    }

    /// Relays recent creature batches from players around `star` to seat `to`.
    pub(super) fn relay_fauna(
        &mut self,
        to: usize,
        (conn, star): (ConnId, Option<StarId>),
        out: &mut Outbox,
    ) {
        let Some(target_star) = star else {
            return;
        };
        for from in 0..self.seats.len() {
            if from == to || self.seats[from].is_none() {
                continue;
            }
            let recent = &self.fauna.recent[from];
            let Some(newest) = recent.newest() else {
                continue;
            };
            let sent = self.fauna.sent[to][from];
            if newest.star != target_star || newest.seq <= sent {
                continue;
            }
            for latest in recent.iter() {
                let due = latest.seq > sent
                    && (latest.seq > self.relayed || latest.head.time == newest.head.time);
                if due {
                    offer(&mut self.batch, latest, conn, out);
                }
            }
            self.fauna.sent[to][from] = newest.seq;
        }
    }
}
