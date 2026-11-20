//! The ring buffer of recent wake sources.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::packet::Packet;
use super::tune::{LIFE, WAKES};

/// One wake source: where it laid its waves, and which way the source went and which way was up.
///
/// The three points are in the frame of the anchor, so that a rebase or the spin of the planet
/// moves them with the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wake {
    /// Where the source stood on the sea.
    pub at: DVec3,
    /// The point one metre ahead of it along its way.
    pub ahead: DVec3,
    /// The point one metre over it.
    pub over: DVec3,
    /// The waves it laid.
    pub packet: Packet,
}

/// The last [`WAKES`] wake sources, the newest over the oldest once the buffer is full.
#[derive(Resource, Debug, Clone)]
pub struct Wakes {
    /// The entries, in the order they were written.
    entries: Vec<Wake>,
    /// The slot the next entry goes to.
    next: usize,
}

impl Default for Wakes {
    /// Returns an empty buffer that never grows past [`WAKES`].
    fn default() -> Self {
        Self {
            entries: Vec::with_capacity(WAKES),
            next: 0,
        }
    }
}

impl Wakes {
    /// Writes `wake` into the next slot, over the oldest entry once the buffer is full.
    pub fn push(&mut self, wake: Wake) {
        match self.entries.len() < WAKES {
            true => self.entries.push(wake),
            false => self.entries[self.next] = wake,
        }
        self.next = (self.next + 1) % WAKES;
    }

    /// Returns the entries that still show at clock `now`.
    pub fn live(&self, now: f32) -> impl Iterator<Item = &Wake> {
        self.entries
            .iter()
            .filter(move |wake| (0.0..LIFE).contains(&(now - wake.packet.born)))
    }

    /// Returns true when some entry still shows at clock `now`.
    pub fn stirred(&self, now: f32) -> bool {
        self.live(now).next().is_some()
    }

    /// Returns how many entries the buffer holds, live or not.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when the buffer holds nothing.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
