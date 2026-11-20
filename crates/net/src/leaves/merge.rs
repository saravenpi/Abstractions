//! Merging one set of leaves into another.

use super::Leaves;

impl Leaves {
    /// Adds every leaf of `other` and returns the ones that were not here.
    pub fn merge(&mut self, other: &Self) -> Self {
        let mut fresh = Self::default();
        for (start, stop) in &other.runs {
            for (from, to) in self.uncovered(*start, *stop) {
                fresh.add(from, to);
            }
        }
        for (start, stop) in &other.runs {
            self.add(*start, *stop);
        }
        fresh
    }

    /// Returns the parts of `start..stop` that the set does not hold.
    fn uncovered(&self, start: u64, stop: u64) -> Vec<(u64, u64)> {
        let mut cursor = start;
        if let Some((_, end)) = self.runs.range(..=start).next_back() {
            cursor = cursor.max(*end);
        }
        let mut gaps = Vec::new();
        for (from, to) in self.runs.range(start..stop) {
            if *from > cursor {
                gaps.push((cursor, *from));
            }
            cursor = cursor.max(*to);
        }
        if cursor < stop {
            gaps.push((cursor, stop));
        }
        gaps
    }
}
