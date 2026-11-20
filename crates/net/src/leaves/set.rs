//! The set of leaves of one body, stored as sorted, disjoint, non-touching runs.

use std::collections::BTreeMap;

use super::valid;

/// A set of leaf keys. Neighbouring keys merge into one run, so a path costs about one run a
/// leaf at worst and less where the path runs along the Morton order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Leaves {
    /// Start of each run and the key after its last.
    pub(super) runs: BTreeMap<u64, u64>,
    /// Number of keys in the set.
    count: u64,
}

impl Leaves {
    /// Returns the number of leaves.
    pub fn len(&self) -> u64 {
        self.count
    }

    /// Returns true when no leaf is in the set.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the number of runs.
    pub fn run_count(&self) -> usize {
        self.runs.len()
    }

    /// Returns true when `leaf` is in the set.
    pub fn contains(&self, leaf: u64) -> bool {
        let before = self.runs.range(..=leaf).next_back();
        before.is_some_and(|(_, end)| leaf < *end)
    }

    /// Adds `leaf` and returns true when it was not there. A key that names no leaf is ignored.
    pub fn insert(&mut self, leaf: u64) -> bool {
        valid(leaf) && self.add(leaf, leaf + 1) > 0
    }

    /// Returns every leaf, lowest key first.
    pub fn iter(&self) -> impl Iterator<Item = u64> + '_ {
        self.runs.iter().flat_map(|(start, stop)| *start..*stop)
    }

    /// Adds the keys `start..stop`, joining every run it touches, and returns how many were new.
    pub(super) fn add(&mut self, start: u64, stop: u64) -> u64 {
        let touching: Vec<(u64, u64)> = self
            .runs
            .range(..=stop)
            .rev()
            .take_while(|(_, end)| **end >= start)
            .map(|(from, to)| (*from, *to))
            .collect();
        let (mut from, mut to, mut held) = (start, stop, 0);
        for (a, b) in touching {
            self.runs.remove(&a);
            held += b.min(stop).saturating_sub(a.max(start));
            (from, to) = (from.min(a), to.max(b));
        }
        self.runs.insert(from, to);
        let fresh = stop - start - held;
        self.count += fresh;
        fresh
    }
}
