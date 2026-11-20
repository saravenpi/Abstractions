//! The leaves of one body.

use abstractions_net::leaves::Leaves;

/// The leaves of one body uncovered so far.
#[derive(Debug, Clone, Default)]
pub(super) struct Ground {
    /// Every leaf uncovered.
    pub(super) leaves: Leaves,
    /// Leaves uncovered here and not yet handed to the network.
    pub(super) new: Leaves,
    /// Grows every time a leaf is uncovered.
    pub(super) version: u64,
}

impl Ground {
    /// Uncovers `leaf` here and keeps it for the network. Returns true when it was not
    /// uncovered. A key that names no leaf is ignored.
    pub(super) fn set(&mut self, leaf: u64) -> bool {
        let fresh = self.leaves.insert(leaf);
        if fresh {
            self.new.insert(leaf);
            self.version += 1;
        }
        fresh
    }

    /// Marks every leaf of `other`, which is not for the network.
    pub(super) fn merge(&mut self, other: &Leaves) {
        let fresh = self.leaves.merge(other);
        self.version += u64::from(!fresh.is_empty());
    }
}
