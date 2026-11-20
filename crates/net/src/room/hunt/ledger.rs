//! How the carcasses and the spears of the hunt are kept, taken from and forgotten.

use super::{HURT_KEEP, Hunting};
use crate::wire::{AnimalKey, CARCASSES_MAX, Carcass, SPEARS_MAX, Spear, SpearKey};

impl Hunting {
    /// Keeps `carcass`, dropping the oldest one and the spears stuck in it when the room is full.
    pub(super) fn lay(&mut self, carcass: Carcass) {
        if self.carcasses.len() >= CARCASSES_MAX {
            let gone = self.carcasses.remove(0);
            self.spears.retain(|spear| spear.animal != gone.animal);
        }
        self.carcasses.push(carcass);
    }

    /// Takes `part` from the carcass of `animal` and returns true when it was still there.
    pub(in crate::room) fn carve(&mut self, animal: AnimalKey, part: u8) -> bool {
        let found = self.carcasses.iter_mut().find(|c| c.animal == animal);
        found.is_some_and(|carcass| {
            let was = carcass.taken;
            carcass.taken |= 1 << part;
            carcass.taken != was
        })
    }

    /// Keeps `spear`, replacing the one with its key and dropping the oldest when full.
    pub(super) fn plant(&mut self, spear: Spear) {
        self.spears.retain(|old| old.key != spear.key);
        if self.spears.len() >= SPEARS_MAX {
            self.spears.remove(0);
        }
        self.spears.push(spear);
    }

    /// Takes the spear `key` out and returns true when it stood.
    pub(in crate::room) fn pull(&mut self, key: SpearKey) -> bool {
        let before = self.spears.len();
        self.spears.retain(|spear| spear.key != key);
        self.spears.len() != before
    }

    /// Forgets the carcasses that lasted their time at universe time `now`, the spears stuck in
    /// them and the wounds nobody remembers. Returns true when a carcass went.
    pub(in crate::room) fn prune(&mut self, now: f64) -> bool {
        self.hurt.retain(|_, hurt| now - hurt.last < HURT_KEEP);
        self.wounds
            .retain(|(_, wound)| now - wound.opened < HURT_KEEP);
        let mut gone = Vec::new();
        self.carcasses.retain(|c| {
            let keep = now < c.since + f64::from(c.lasts);
            if !keep {
                gone.push(c.animal);
            }
            keep
        });
        self.spears.retain(|spear| !gone.contains(&spear.animal));
        !gone.is_empty()
    }
}
