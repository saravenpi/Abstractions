//! The hotbar: six slots that each point at an item of the pack, one of them held.

use serde::{Deserialize, Serialize};

/// Number of hotbar slots.
pub const SLOTS: usize = 6;

/// Which item each slot points at, by item id, and which slot is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Bar {
    /// The item id in each slot.
    #[serde(with = "super::slots")]
    pub slots: [Option<u32>; SLOTS],
    /// The held slot.
    pub held: usize,
}

impl Bar {
    /// Returns the id of the held item, if its slot holds one.
    pub fn held(&self) -> Option<u32> {
        self.slots.get(self.held).copied().flatten()
    }

    /// Holds the slot `slot`, if there is one.
    pub fn select(&mut self, slot: usize) {
        if slot < SLOTS {
            self.held = slot;
        }
    }

    /// Puts item `id` in `slot`, taking it out of any other slot.
    pub fn put(&mut self, slot: usize, id: u32) {
        if slot < SLOTS {
            self.forget(id);
            self.slots[slot] = Some(id);
        }
    }

    /// Puts item `id` in the first empty slot, unless a slot already points at it.
    pub fn place(&mut self, id: u32) {
        if self.slots.contains(&Some(id)) {
            return;
        }
        if let Some(free) = self.slots.iter_mut().find(|s| s.is_none()) {
            *free = Some(id);
        }
    }

    /// Empties every slot that points at item `id`.
    pub fn forget(&mut self, id: u32) {
        self.slots
            .iter_mut()
            .filter(|s| **s == Some(id))
            .for_each(|s| *s = None);
    }
}
