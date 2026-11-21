//! Claims: alone a cut part goes into the pack at once; in a session it waits for the server, which
//! gives each part of a carcass to the first player who cuts it.

use bevy::prelude::*;

use super::Cut;

/// The cuts that wait for the answer of the server.
#[derive(Resource, Debug, Default)]
pub struct Claims {
    /// True while a server decides who gets each part.
    pub remote: bool,
    /// The cuts asked and not answered yet: the carcass key, the part and the cut.
    pub pending: Vec<(u64, u8, Cut)>,
}

impl Claims {
    /// Returns the cut to harvest now: at once alone; nothing in a session, where it waits under the
    /// carcass `key` and its `part`.
    pub fn claim(&mut self, (key, part): (u64, u8), cut: Cut) -> Option<Cut> {
        if !self.remote {
            return Some(cut);
        }
        self.pending.push((key, part, cut));
        None
    }

    /// Takes out the cut of `part` of the carcass `key` that the server answered, if one waits.
    pub fn answered(&mut self, (key, part): (u64, u8)) -> Option<Cut> {
        let at = self
            .pending
            .iter()
            .position(|(k, p, _)| *k == key && *p == part)?;
        Some(self.pending.remove(at).2)
    }

    /// Decides alone again, and returns the cuts the server never answered, to harvest now.
    pub fn release(&mut self) -> Vec<Cut> {
        self.remote = false;
        self.pending.drain(..).map(|(_, _, cut)| cut).collect()
    }
}
