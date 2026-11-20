//! The kill ledger: members of herds killed by players, each gone from its herd until its death time
//! plus [`RECOVERY`].
//!
//! Herds are pure functions of the seed, the herd and universe time; a kill by a player is the one
//! thing that is not. The ledger holds those kills as `(herd, slot, time of death)`, so a herd is its
//! pure function minus the ledger. A member is back, as a new one born into the herd, once its
//! recovery is over, so the ledger clears itself. The server will hold it; for now each machine
//! keeps its own.

use crate::life::hunt::RECOVERY;

/// One kill: the herd, the slot of the member and the universe time it died.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kill {
    /// Identity of the herd.
    pub herd: u64,
    /// Slot of the member.
    pub slot: u8,
    /// Universe time of the death in seconds.
    pub at: f64,
}

/// The kills by players whose recovery is not over.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ledger {
    /// The kills, in the order they were recorded.
    kills: Vec<Kill>,
}

impl Ledger {
    /// Records the death of member `slot` of herd `herd` at universe time `at`, once.
    pub fn kill(&mut self, herd: u64, slot: u8, at: f64) {
        if !self.dead(herd, slot, at) {
            self.kills.push(Kill { herd, slot, at });
        }
    }

    /// Returns true when member `slot` of herd `herd` is dead by the ledger at universe `time`.
    pub fn dead(&self, herd: u64, slot: u8, time: f64) -> bool {
        self.kills
            .iter()
            .any(|k| k.herd == herd && k.slot == slot && k.at <= time && time < k.at + RECOVERY)
    }

    /// Forgets the kills whose recovery is over at universe `time`.
    pub fn prune(&mut self, time: f64) {
        self.kills.retain(|k| time < k.at + RECOVERY);
    }

    /// Returns the kills still held.
    pub fn kills(&self) -> &[Kill] {
        &self.kills
    }
}
