//! The bitset of one body.

use super::grid::{CELLS, WORDS};

/// The cells of one body uncovered so far.
#[derive(Debug, Clone)]
pub(super) struct Ground {
    /// One bit a cell, [`WORDS`] words.
    pub(super) words: Vec<u64>,
    /// Cells uncovered here and not yet handed to the network.
    pub(super) new: Vec<u32>,
    /// Grows every time a cell is uncovered.
    pub(super) version: u64,
}

impl Default for Ground {
    fn default() -> Self {
        Self {
            words: vec![0; WORDS],
            new: Vec::new(),
            version: 0,
        }
    }
}

impl Ground {
    /// Sets the bit of `cell` and returns true when it was not set. Cells off the grid are
    /// ignored.
    pub(super) fn set(&mut self, cell: u32) -> bool {
        if cell as usize >= CELLS {
            return false;
        }
        let (word, bit) = (cell as usize / 64, 1u64 << (cell % 64));
        let fresh = self.words[word] & bit == 0;
        self.words[word] |= bit;
        self.version += u64::from(fresh);
        fresh
    }

    /// Sets every bit set in `words`. Words past the grid are ignored.
    pub(super) fn merge(&mut self, words: &[u64]) {
        let mut fresh = false;
        for (mine, theirs) in self.words.iter_mut().zip(words) {
            fresh |= *theirs & !*mine != 0;
            *mine |= theirs;
        }
        self.version += u64::from(fresh);
    }
}
