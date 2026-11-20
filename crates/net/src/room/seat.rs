//! A seat: one player in the room and the latest states they sent of each kind.

use crate::host::ConnId;
use crate::ids::StarId;
use crate::wire::{BODY_MAX, Head, Kind, PlayerInfo, Record, Waypoint};

/// Most records of one player and kind the room keeps between two ticks.
pub(super) const DEPTH: usize = 4;

/// The latest state record of one player and kind, kept as bytes the room never decodes.
#[derive(Debug)]
pub(super) struct Latest {
    /// The head as the player sent it.
    pub(super) head: Head,
    /// The body as the player sent it. Its memory is reused for every newer record.
    pub(super) body: Vec<u8>,
    /// The star of the record's frame.
    pub(super) star: StarId,
    /// Number of the record in the room: higher is newer.
    pub(super) seq: u64,
}

/// The last [`DEPTH`] records of one player and kind, oldest first, in slots whose memory is
/// reused once all are taken.
#[derive(Debug, Default)]
pub(super) struct Recent {
    /// The records, at most [`DEPTH`].
    slots: Vec<Latest>,
    /// The slot of the oldest record once every slot is taken.
    start: usize,
}

impl Recent {
    /// Returns the records oldest first.
    pub(super) fn iter(&self) -> impl Iterator<Item = &Latest> {
        let (newer, older) = self.slots.split_at(self.start);
        older.iter().chain(newer)
    }

    /// Returns the newest record, if any.
    pub(super) fn newest(&self) -> Option<&Latest> {
        let last = match self.start {
            0 => self.slots.len().checked_sub(1)?,
            start => start - 1,
        };
        self.slots.get(last)
    }

    /// Returns the slot for a new record: a new one while fewer than [`DEPTH`] are taken, else
    /// the oldest, which becomes the newest.
    fn next_slot(&mut self, head: Head) -> &mut Latest {
        if self.slots.len() < DEPTH {
            self.slots.push(Latest {
                head,
                body: Vec::with_capacity(BODY_MAX),
                star: head.frame.star(),
                seq: 0,
            });
            let last = self.slots.len() - 1;
            return &mut self.slots[last];
        }
        let slot = self.start;
        self.start = (self.start + 1) % DEPTH;
        &mut self.slots[slot]
    }
}

/// What a recipient last got of one player and kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Sent {
    /// Number of the record sent, 0 when none was.
    pub(super) seq: u64,
    /// Network time it was sent.
    pub(super) at: f64,
}

impl Sent {
    /// Nothing sent yet.
    pub(super) const NONE: Self = Self {
        seq: 0,
        at: f64::NEG_INFINITY,
    };
}

/// One player in the room.
#[derive(Debug)]
pub(super) struct Seat {
    /// The player's connection.
    pub(super) conn: ConnId,
    /// The player's identity key.
    pub(super) key: [u8; 32],
    /// The player's id and cleaned name.
    pub(super) info: PlayerInfo,
    /// The star of the player's latest record, or nothing before the first one.
    pub(super) star: Option<StarId>,
    /// The recent records of each kind, in the order of [`Kind::index`].
    pub(super) recent: [Recent; Kind::COUNT],
    /// The player's live waypoint, if any.
    pub(super) waypoint: Option<Waypoint>,
    /// The bytes of the item the player holds, if any.
    pub(super) held: Option<Vec<u8>>,
}

impl Seat {
    /// Returns the seat of a player who just joined, with no state yet.
    pub(super) fn new(conn: ConnId, key: [u8; 32], info: PlayerInfo) -> Self {
        Self {
            conn,
            key,
            info,
            star: None,
            recent: Default::default(),
            waypoint: None,
            held: None,
        }
    }

    /// Keeps `record` as the newest of its kind under number `seq` and returns true. A body
    /// over [`BODY_MAX`] bytes, a time that is not a number, or a sample older than the newest
    /// kept is dropped and false is returned.
    pub(super) fn keep(&mut self, record: &Record<'_>, seq: u64) -> bool {
        let recent = &mut self.recent[record.head.kind.index()];
        let older = recent
            .newest()
            .is_some_and(|newest| record.head.time < newest.head.time);
        if record.body.len() > BODY_MAX || !record.head.time.is_finite() || older {
            return false;
        }
        let star = record.head.frame.star();
        let latest = recent.next_slot(record.head);
        latest.head = record.head;
        latest.body.clear();
        latest.body.extend_from_slice(record.body);
        latest.star = star;
        latest.seq = seq;
        self.star = Some(star);
        true
    }
}
