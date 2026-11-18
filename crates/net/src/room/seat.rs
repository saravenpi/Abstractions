//! A seat: one player in the room and the latest state they sent of each kind.

use crate::host::ConnId;
use crate::ids::StarId;
use crate::wire::{Head, Kind, PlayerInfo, Record};

/// Kinds of state record: robot, ship and companion.
pub(super) const KINDS: usize = 3;

/// Most bytes of a state body the room keeps. A longer body is dropped.
pub(super) const BODY_MAX: usize = 128;

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
    /// The latest record of each kind, in the order of [`kind_index`].
    pub(super) latest: [Option<Latest>; KINDS],
}

/// Returns the place of `kind` in [`Seat::latest`].
pub(super) fn kind_index(kind: Kind) -> usize {
    match kind {
        Kind::Robot => 0,
        Kind::Ship => 1,
        Kind::Companion => 2,
    }
}

impl Seat {
    /// Returns the seat of a player who just joined, with no state yet.
    pub(super) fn new(conn: ConnId, key: [u8; 32], info: PlayerInfo) -> Self {
        Self {
            conn,
            key,
            info,
            star: None,
            latest: [None, None, None],
        }
    }

    /// Keeps `record` as the latest of its kind under number `seq` and returns true. A body
    /// over [`BODY_MAX`] bytes, a time that is not a number, or a sample older than the one
    /// kept is dropped and false is returned.
    pub(super) fn keep(&mut self, record: &Record<'_>, seq: u64) -> bool {
        let slot = &mut self.latest[kind_index(record.head.kind)];
        let older = slot
            .as_ref()
            .is_some_and(|latest| record.head.time < latest.head.time);
        if record.body.len() > BODY_MAX || !record.head.time.is_finite() || older {
            return false;
        }
        let star = record.head.frame.star();
        let latest = slot.get_or_insert_with(|| Latest {
            head: record.head,
            body: Vec::with_capacity(BODY_MAX),
            star,
            seq,
        });
        latest.head = record.head;
        latest.body.clear();
        latest.body.extend_from_slice(record.body);
        latest.star = star;
        latest.seq = seq;
        self.star = Some(star);
        true
    }
}
