//! Lending a tool out of the pack: a thrown spear leaves the pack while it flies and comes back
//! to its hotbar slot when it is picked up.

use super::{Item, Pack, SLOTS};

/// A tool taken out of the pack, with the hotbar slot it stood in.
pub type Lent = (Item, Option<usize>);

impl Pack {
    /// Takes the tool with `id` out of the pack and its hotbar slot, and returns it with that slot.
    pub fn lend(&mut self, id: u32) -> Option<Lent> {
        let at = self.items.iter().position(|item| item.id == id)?;
        let slot = self.bar.slots.iter().position(|s| *s == Some(id));
        self.bar.forget(id);
        Some((self.items.remove(at), slot))
    }

    /// Puts a lent tool back, in the slot it stood in when that is empty, else in a free one.
    pub fn restore(&mut self, (item, slot): Lent) {
        let id = item.id;
        self.items.push(item);
        match slot.filter(|s| *s < SLOTS && self.bar.slots[*s].is_none()) {
            Some(slot) => self.bar.slots[slot] = Some(id),
            None => self.bar.place(id),
        }
    }
}
