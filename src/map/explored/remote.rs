//! The explored ground that crosses the network: runs from the server, and the leaves
//! uncovered here waiting to be sent.

use abstractions_net::leaves::{Leaves, Run};
use abstractions_net::room::BODIES_MAX;

use super::Explored;
use super::ground::Ground;
use crate::universe::BodyId;

impl Explored {
    /// Returns the ground of `body`, made blank on first use, or nothing once [`BODIES_MAX`]
    /// bodies are known: a server cannot make this game keep more.
    pub(super) fn ground(&mut self, body: BodyId) -> Option<&mut Ground> {
        let full = self.bodies.len() >= BODIES_MAX && !self.bodies.contains_key(&body);
        (!full).then(|| self.bodies.entry(body).or_default())
    }

    /// Marks the leaves in `runs`, as another player, a server or a save holds them. They are
    /// not sent back to the server. Keys that name no leaf are ignored, and a list without any
    /// leaf keeps nothing.
    pub fn merge_runs(&mut self, body: BodyId, runs: &[Run]) {
        let leaves = Leaves::from_runs(runs);
        if leaves.is_empty() {
            return;
        }
        if let Some(ground) = self.ground(body) {
            ground.merge(&leaves);
        }
    }

    /// Returns the leaves of `body` uncovered here since the last call, for the network to send.
    pub fn take_new(&mut self, body: BodyId) -> Leaves {
        self.bodies
            .get_mut(&body)
            .map(|ground| std::mem::take(&mut ground.new))
            .unwrap_or_default()
    }

    /// Forgets the leaves uncovered here and not yet sent: without a server they would pile up.
    pub fn drop_unsent(&mut self) {
        for ground in self.bodies.values_mut() {
            ground.new = Leaves::default();
        }
    }
}

#[cfg(test)]
#[path = "remote_tests.rs"]
mod tests;
