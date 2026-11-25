//! Wearable armor equipment, pack armor operations, and energy shield capacity synchronization.

pub mod spec;
#[cfg(test)]
mod tests;

pub use spec::{Armor, ArmorSlot};

use bevy::prelude::*;

use crate::condition::{Part, Shield};
use crate::craft::matter::Mineral;
use crate::craft::pack::{BAR_SLOTS, Pack, SlotEntry, TOTAL_SLOTS};

impl Pack {
    /// Returns the armor currently equipped in `slot`, if any.
    pub fn equipped_armor(&self, slot: ArmorSlot) -> Option<&Armor> {
        let id = self.equipped[slot as usize]?;
        self.armor(id)
    }

    /// Returns the physical damage reduction fraction (`0.0..=0.84`) covering `part`.
    pub fn defense_for(&self, part: Part) -> f32 {
        ArmorSlot::ALL
            .into_iter()
            .find(|slot| slot.covers(part))
            .and_then(|slot| self.equipped_armor(slot))
            .map_or(0.0, Armor::defense)
    }

    /// Returns the combined maximum energy shield capacity in joules across all equipped pieces.
    pub fn total_shield_max(&self) -> f32 {
        ArmorSlot::ALL
            .into_iter()
            .filter_map(|slot| self.equipped_armor(slot))
            .map(Armor::shield_joules)
            .sum()
    }

    /// Creates a new armor piece in the first free pack or hotbar slot and returns its id.
    pub fn add_armor(&mut self, slot: ArmorSlot, mineral: Mineral, shielded: bool) -> Option<u32> {
        self.ensure_grid();
        let free = (BAR_SLOTS..TOTAL_SLOTS)
            .chain(0..BAR_SLOTS)
            .find(|&i| self.grid[i].is_none())?;
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        self.armors.push(Armor {
            id,
            slot,
            mineral,
            shielded,
            wear: 0.0,
        });
        self.grid[free] = Some(SlotEntry::Armor(id));
        self.sync_views();
        Some(id)
    }

    /// Equips the armor item stored at `slot_idx`, swapping any previously equipped piece back.
    pub fn equip_armor_from_slot(&mut self, slot_idx: usize) -> bool {
        self.ensure_grid();
        let Some(SlotEntry::Armor(id)) = self.grid.get(slot_idx).copied().flatten() else {
            return false;
        };
        let Some(target) = self.armor(id).map(|a| a.slot) else {
            return false;
        };
        let prev = self.equipped[target as usize].replace(id);
        self.grid[slot_idx] = prev.map(SlotEntry::Armor);
        self.sync_views();
        true
    }

    /// Unequips the armor in `slot` into the first free pack or hotbar slot.
    pub fn unequip_armor(&mut self, slot: ArmorSlot) -> bool {
        self.ensure_grid();
        let Some(id) = self.equipped[slot as usize] else {
            return false;
        };
        let Some(free) = (BAR_SLOTS..TOTAL_SLOTS)
            .chain(0..BAR_SLOTS)
            .find(|&i| self.grid[i].is_none())
        else {
            return false;
        };
        self.equipped[slot as usize] = None;
        self.grid[free] = Some(SlotEntry::Armor(id));
        self.sync_views();
        true
    }
}

/// Synchronizes [`Shield::max`] from the currently equipped Aegis armor in [`Pack`].
pub fn sync_shield(pack: Res<Pack>, mut shield: ResMut<Shield>) {
    shield.set_max(pack.total_shield_max());
}

/// Registers armor equipment shield synchronization.
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        sync_shield
            .run_if(resource_exists::<Shield>)
            .run_if(resource_changed::<Pack>.or_else(resource_added::<Shield>)),
    );
}
