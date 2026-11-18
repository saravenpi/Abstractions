//! The snapshots of one body in time order, in a fixed ring that never allocates.

use super::{Snap, View};

/// Most snapshots a buffer keeps: a second at the full rate.
pub const CAPACITY: usize = 32;

/// The snapshots in time order, the oldest overwritten first once the ring is full.
#[derive(Debug, Clone, Default)]
pub(super) struct Ring<E> {
    /// The slots.
    snaps: [Snap<E>; CAPACITY],
    /// The slot of the oldest snapshot.
    start: usize,
    /// How many slots hold a snapshot.
    len: usize,
}

impl<E: Copy> Ring<E> {
    /// Keeps `snap` and returns true when its time is finite and later than the newest one.
    /// A late or repeated snapshot is dropped and false is returned.
    pub(super) fn push(&mut self, snap: Snap<E>) -> bool {
        let fresh =
            snap.time.is_finite() && self.newest().is_none_or(|newest| snap.time > newest.time);
        if !fresh {
            return false;
        }
        self.snaps[(self.start + self.len) % CAPACITY] = snap;
        if self.len == CAPACITY {
            self.start = (self.start + 1) % CAPACITY;
        } else {
            self.len += 1;
        }
        true
    }

    /// Returns the snapshot `index` places after the oldest.
    fn get(&self, index: usize) -> &Snap<E> {
        &self.snaps[(self.start + index) % CAPACITY]
    }

    /// Returns the newest snapshot.
    fn newest(&self) -> Option<&Snap<E>> {
        self.len.checked_sub(1).map(|last| self.get(last))
    }

    /// Forgets every snapshot.
    pub(super) fn clear(&mut self) {
        self.start = 0;
        self.len = 0;
    }

    /// Returns how many snapshots are kept.
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// Returns the state at `time`: the oldest snapshot before it, the newest carried forward
    /// after it, and the blend of the two snapshots around it in between.
    pub(super) fn at(&self, time: f64) -> Option<View<E>> {
        let newest = self.newest()?;
        if time >= newest.time {
            return Some(newest.ahead(time));
        }
        let oldest = self.get(0);
        if time <= oldest.time {
            return Some(oldest.hold(time));
        }
        let after = (1..self.len).find(|&index| self.get(index).time > time)?;
        Some(self.get(after - 1).toward(self.get(after), time))
    }
}
