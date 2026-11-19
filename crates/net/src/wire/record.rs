//! State records and the batches a datagram carries.
//!
//! A batch is records one after the other, with no count: read until the bytes run out. A
//! record is a [`Head`] and a body. The body is opaque here: the server keeps the last bytes of
//! each player and kind and copies them to the others without reading them.

use serde::{Deserialize, Serialize};

use super::{Frame, Kind};
use crate::log::PlayerId;

/// Bit of [`Head::flags`] set when the thing is at rest, so it is sent at the slow rate.
pub const AT_REST: u8 = 1;

/// The readable part of a record: whose it is, what it is, and where and when it was sampled.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Head {
    /// The player the thing belongs to. A server overwrites it with the sender's id.
    pub player: PlayerId,
    /// What the body describes.
    pub kind: Kind,
    /// Frame the body is expressed in.
    pub frame: Frame,
    /// Universe time of the sample in seconds.
    pub time: f64,
    /// Bits such as [`AT_REST`]. Unknown bits are kept and ignored.
    pub flags: u8,
}

impl Head {
    /// Returns true when the thing is at rest.
    pub fn at_rest(&self) -> bool {
        self.flags & AT_REST != 0
    }
}

/// One state record: a head anyone reads and a body only players read.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Record<'a> {
    /// The readable part.
    pub head: Head,
    /// The packed state, as one of the body types of this module wrote it for `head.kind`.
    pub body: &'a [u8],
}

/// Adds `record` at the end of `batch`.
pub fn push(batch: &mut Vec<u8>, record: &Record<'_>) {
    let start = batch.len();
    if postcard::to_io(record, &mut *batch).is_err() {
        batch.truncate(start);
    }
}

/// Adds `record` at the end of `batch` and returns true, unless the batch would grow past
/// `max` bytes: then the batch is left as it was and false is returned.
pub fn push_within(batch: &mut Vec<u8>, record: &Record<'_>, max: usize) -> bool {
    let start = batch.len();
    push(batch, record);
    let fits = batch.len() > start && batch.len() <= max;
    if !fits {
        batch.truncate(start);
    }
    fits
}

/// Returns the batches that carry `records` in order, none longer than `max` bytes, as few as
/// that order allows. Give it the connection's largest datagram size. A record that is longer
/// than `max` alone is left out.
pub fn split<'a>(records: impl IntoIterator<Item = Record<'a>>, max: usize) -> Vec<Vec<u8>> {
    let mut batches = vec![Vec::new()];
    for record in records {
        let last = batches.len() - 1;
        if push_within(&mut batches[last], &record, max) {
            continue;
        }
        let mut next = Vec::new();
        if push_within(&mut next, &record, max) {
            batches.push(next);
        }
    }
    batches.retain(|batch| !batch.is_empty());
    batches
}

/// The records of a received batch, read one at a time without decoding their bodies.
#[derive(Debug, Clone)]
pub struct Records<'a> {
    /// The bytes not read yet.
    rest: &'a [u8],
}

/// Returns the records of `batch` in order. Reading stops at the first record that is cut
/// short or malformed.
pub fn records(batch: &[u8]) -> Records<'_> {
    Records { rest: batch }
}

impl<'a> Iterator for Records<'a> {
    type Item = Record<'a>;

    /// Returns the next record of the batch, or nothing at its end or at a malformed record.
    fn next(&mut self) -> Option<Record<'a>> {
        let (record, rest) = postcard::take_from_bytes::<Record<'a>>(self.rest).ok()?;
        self.rest = rest;
        Some(record)
    }
}
