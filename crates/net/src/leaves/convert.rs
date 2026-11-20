//! Leaves to runs and back: the form they travel and are stored in.

use super::{Leaves, Run, parts, valid};

impl Leaves {
    /// Returns the runs, each placed from the end of the one before.
    pub fn runs(&self) -> Vec<Run> {
        let mut end = 0u64;
        self.runs
            .iter()
            .map(|(start, stop)| {
                let gap = start - end;
                end = *stop;
                Run {
                    gap,
                    len: stop - start,
                }
            })
            .collect()
    }

    /// Returns the runs in groups of at most `size`, each group placed from 0 on its own, so
    /// each can travel alone.
    pub fn chunks(&self, size: usize) -> Vec<Vec<Run>> {
        let all: Vec<(u64, u64)> = self.runs.iter().map(|(a, b)| (*a, *b)).collect();
        all.chunks(size.max(1)).map(place).collect()
    }

    /// Returns the set of `runs`. A run that overflows, or holds a key that names no leaf, is
    /// left out.
    pub fn from_runs(runs: &[Run]) -> Self {
        let mut leaves = Self::default();
        let mut end = 0u64;
        for run in runs {
            let Some(start) = end.checked_add(run.gap) else {
                break;
            };
            let Some(stop) = start.checked_add(run.len) else {
                break;
            };
            end = stop;
            if run.len > 0 && valid(start) && same_block(start, stop - 1) {
                leaves.add(start, stop);
            }
        }
        leaves
    }
}

/// Returns `part`, runs as start and stop keys, as runs placed from 0.
fn place(part: &[(u64, u64)]) -> Vec<Run> {
    let mut end = 0;
    let mut runs = Vec::with_capacity(part.len());
    for (start, stop) in part {
        runs.push(Run {
            gap: start - end,
            len: stop - start,
        });
        end = *stop;
    }
    runs
}

/// Returns true when both keys share a face and a depth, so every key between them is a leaf.
fn same_block(first: u64, last: u64) -> bool {
    let (a, b) = (parts(first), parts(last));
    valid(last) && (a.0, a.1) == (b.0, b.1)
}
