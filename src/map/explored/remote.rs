//! The explored ground that crosses the network: cells and bitsets from the server, and the
//! cells uncovered here waiting to be sent.

use abstractions_net::room::BODIES_MAX;

use super::Explored;
use super::grid::{CELLS, WORDS};
use super::ground::Ground;
use crate::universe::BodyId;

impl Explored {
    /// Returns the ground of `body`, made blank on first use, or nothing once [`BODIES_MAX`]
    /// bodies are known: a server cannot make this game keep more.
    pub(super) fn ground(&mut self, body: BodyId) -> Option<&mut Ground> {
        let full = self.bodies.len() >= BODIES_MAX && !self.bodies.contains_key(&body);
        (!full).then(|| self.bodies.entry(body).or_default())
    }

    /// Marks cells that another player uncovered. They are not sent back to the server. Cells
    /// off the grid are ignored, and a list without any keeps nothing.
    pub fn reveal_cells(&mut self, body: BodyId, cells: &[u32]) {
        let mut cells = cells
            .iter()
            .filter(|cell| (**cell as usize) < CELLS)
            .peekable();
        if cells.peek().is_none() {
            return;
        }
        let Some(ground) = self.ground(body) else {
            return;
        };
        for &cell in cells {
            ground.set(cell);
        }
    }

    /// Marks every cell set in `words`, a whole bitset as the server or a save holds it. A
    /// list of another length, or with no cell set, is ignored.
    pub fn merge_words(&mut self, body: BodyId, words: &[u64]) {
        if words.len() != WORDS || words.iter().all(|word| *word == 0) {
            return;
        }
        if let Some(ground) = self.ground(body) {
            ground.merge(words);
        }
    }

    /// Returns the cells of `body` uncovered here since the last call, for the network to send.
    pub fn take_new(&mut self, body: BodyId) -> Vec<u32> {
        self.bodies
            .get_mut(&body)
            .map(|ground| std::mem::take(&mut ground.new))
            .unwrap_or_default()
    }

    /// Forgets the cells uncovered here and not yet sent: without a server they would pile up.
    pub fn drop_unsent(&mut self) {
        for ground in self.bodies.values_mut() {
            ground.new.clear();
        }
    }
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
