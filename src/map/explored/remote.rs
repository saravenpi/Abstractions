//! The explored ground that crosses the network: cells and bitsets from the server, and the
//! cells uncovered here waiting to be sent.

use super::Explored;
use crate::universe::BodyId;

impl Explored {
    /// Marks cells that another player uncovered. They are not sent back to the server.
    pub fn reveal_cells(&mut self, body: BodyId, cells: &[u32]) {
        let ground = self.bodies.entry(body).or_default();
        for &cell in cells {
            ground.set(cell);
        }
    }

    /// Marks every cell set in `words`, a whole bitset as the server or a save holds it.
    pub fn merge_words(&mut self, body: BodyId, words: &[u64]) {
        self.bodies.entry(body).or_default().merge(words);
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
