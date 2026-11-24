//! The 24+6 slot grid: six hotbar slots (`0..6`) and a 6x4 pack grid (`6..30`), stacking materials up to 64.

use serde::{Deserialize, Serialize};

use super::Pack;
use super::tune::CRUMB;
use crate::craft::matter::{Material, Stuff};

/// Number of hotbar slots at the front of the grid (`0..6`).
pub const BAR_SLOTS: usize = 6;

/// Number of storage slots in the 6x4 pack grid (`6..30`).
pub const GRID_SLOTS: usize = 24;

/// Total inventory slots held in [`Pack::grid`].
pub const TOTAL_SLOTS: usize = BAR_SLOTS + GRID_SLOTS;

/// Maximum discrete items in one stack slot, matching the grid style.
pub const STACK_MAX: u16 = 64;

/// Contents of a single inventory slot.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SlotEntry {
    /// Crafted tool referencing [`Pack::items`] by id.
    Tool(u32),
    /// Crafted firearm referencing [`Pack::guns`] by id.
    Gun(u32),
    /// Stack of up to [`STACK_MAX`] materials, bullets, powder, wood, or placeables.
    Stack {
        /// What the stack holds.
        stuff: Stuff,
        /// Mechanical properties sampled when harvested/crafted.
        material: Material,
        /// Unit count in `1..=STACK_MAX`.
        count: u16,
    },
}

/// Sparse slot representation so TOML never serializes null elements.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlacedSlot {
    /// Zero-based slot index in `0..TOTAL_SLOTS`.
    pub slot: u8,
    /// Item or stack stored at `slot`.
    pub entry: SlotEntry,
}

/// Sparse TOML-safe serializer for `[Option<SlotEntry>; TOTAL_SLOTS]`.
pub mod serde_grid {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{PlacedSlot, STACK_MAX, SlotEntry, TOTAL_SLOTS};

    /// Writes only occupied slots as `PlacedSlot` entries.
    pub fn serialize<S: Serializer>(
        grid: &[Option<SlotEntry>; TOTAL_SLOTS],
        to: S,
    ) -> Result<S::Ok, S::Error> {
        let sparse: Vec<PlacedSlot> = grid
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| {
                slot.map(|entry| PlacedSlot {
                    slot: i as u8,
                    entry,
                })
            })
            .collect();
        sparse.serialize(to)
    }

    /// Reads sparse slot entries back into `[Option<SlotEntry>; TOTAL_SLOTS]`.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        from: D,
    ) -> Result<[Option<SlotEntry>; TOTAL_SLOTS], D::Error> {
        let sparse = Vec::<PlacedSlot>::deserialize(from)?;
        let mut grid = [None; TOTAL_SLOTS];
        for placed in sparse {
            let idx = usize::from(placed.slot);
            if idx < TOTAL_SLOTS {
                grid[idx] = Some(match placed.entry {
                    SlotEntry::Stack {
                        stuff,
                        material,
                        count,
                    } => SlotEntry::Stack {
                        stuff,
                        material,
                        count: count.clamp(1, STACK_MAX),
                    },
                    other => other,
                });
            }
        }
        Ok(grid)
    }
}

/// Converts a legacy mass heap into discrete units (`>= 1`).
pub(super) fn units_from_mass(stuff: &Stuff, mass: f32) -> u16 {
    let unit = stuff.unit_mass().max(1e-4);
    (mass / unit).round().max(1.0).min(f32::from(u16::MAX)) as u16
}

impl Pack {
    /// Migrates legacy `items` and `stacks` TOML data into `grid[0..30]` when `grid` is empty.
    pub fn ensure_grid(&mut self) {
        let empty = self.grid.iter().all(Option::is_none);
        if !empty || (self.items.is_empty() && self.guns.is_empty() && self.stacks.is_empty()) {
            return;
        }
        self.migrate_tools_and_guns();
        self.migrate_legacy_stacks();
        self.sync_views();
    }

    /// Moves existing `items` and `guns` into `grid`, honoring existing hotbar slot bindings.
    fn migrate_tools_and_guns(&mut self) {
        for (i, id) in self.bar.slots.into_iter().enumerate() {
            if let Some(id) = id.filter(|id| self.items.iter().any(|it| it.id == *id)) {
                self.grid[i] = Some(SlotEntry::Tool(id));
            }
        }
        let item_ids: Vec<u32> = self.items.iter().map(|i| i.id).collect();
        for id in item_ids {
            let bound = self
                .grid
                .iter()
                .any(|s| matches!(s, Some(SlotEntry::Tool(has)) if *has == id));
            if !bound && let Some(slot) = self.grid.iter_mut().find(|s| s.is_none()) {
                *slot = Some(SlotEntry::Tool(id));
            }
        }
        let gun_ids: Vec<u32> = self.guns.iter().map(|g| g.id).collect();
        for id in gun_ids {
            if let Some(slot) = self.grid.iter_mut().find(|s| s.is_none()) {
                *slot = Some(SlotEntry::Gun(id));
            }
        }
    }

    /// Splits legacy continuous-kg `stacks` across discrete 64-item grid slots.
    fn migrate_legacy_stacks(&mut self) {
        let legacy = std::mem::take(&mut self.stacks);
        for stack in legacy {
            if stack.mass < CRUMB {
                continue;
            }
            let mut left = units_from_mass(&stack.stuff, stack.mass);
            let slots = (BAR_SLOTS..TOTAL_SLOTS).chain(0..BAR_SLOTS);
            for idx in slots {
                if left == 0 {
                    break;
                }
                if self.grid[idx].is_none() {
                    let put = left.min(STACK_MAX);
                    self.grid[idx] = Some(SlotEntry::Stack {
                        stuff: stack.stuff,
                        material: stack.material,
                        count: put,
                    });
                    left -= put;
                }
            }
        }
    }

    /// Applies any direct mutations made via legacy `bar.slots` back into `grid[0..BAR_SLOTS]`.
    pub(super) fn sync_bar_into_grid(&mut self) {
        for i in 0..BAR_SLOTS {
            let Some(id) = self.bar.slots[i] else {
                continue;
            };
            if matches!(self.grid[i], Some(SlotEntry::Tool(has)) if has == id) {
                continue;
            }
            if let Some(prev) = self
                .grid
                .iter()
                .position(|s| matches!(s, Some(SlotEntry::Tool(has)) if *has == id))
            {
                self.grid.swap(i, prev);
            } else if self.items.iter().any(|it| it.id == id) {
                if let Some(displaced) = self.grid[i].take()
                    && let Some(free) = self.grid.iter_mut().find(|s| s.is_none())
                {
                    *free = Some(displaced);
                }
                self.grid[i] = Some(SlotEntry::Tool(id));
            }
        }
    }

    /// Rebuilds compatibility `bar.slots` and grouped `stacks` views from authoritative `grid`.
    pub fn sync_views(&mut self) {
        for i in 0..BAR_SLOTS {
            self.bar.slots[i] = match self.grid[i] {
                Some(SlotEntry::Tool(id)) => Some(id),
                _ => None,
            };
        }
        self.stacks.clear();
        for slot in self.grid.iter().flatten() {
            let SlotEntry::Stack {
                stuff,
                material,
                count,
            } = *slot
            else {
                continue;
            };
            let mass = f32::from(count) * stuff.unit_mass();
            match self.stacks.iter_mut().find(|s| s.stuff == stuff) {
                Some(heap) => heap.mass += mass,
                None => self.stacks.push(super::Stack {
                    stuff,
                    material,
                    mass,
                }),
            }
        }
    }
}
