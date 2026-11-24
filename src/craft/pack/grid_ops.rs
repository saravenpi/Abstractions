//! Slot grid operations: adding, draining, swapping, splitting, quick-moving, and sorting slots.

use super::slot::{BAR_SLOTS, GRID_SLOTS, STACK_MAX, SlotEntry, TOTAL_SLOTS};
use super::{Item, Pack};
use crate::craft::gun::{Gun, GunKind};
use crate::craft::matter::{Material, Mineral, Stuff};

/// Returns sort rank and label so pack sorting groups items predictably by kind.
fn sort_key(pack: &Pack, entry: &SlotEntry) -> (u8, String) {
    match *entry {
        SlotEntry::Tool(id) => (0, pack.item(id).map_or_else(String::new, Item::name)),
        SlotEntry::Gun(id) => (1, pack.gun(id).map_or_else(String::new, Gun::name)),
        SlotEntry::Stack { stuff, .. } => {
            let group = match stuff {
                Stuff::Placeable(_) => 2,
                Stuff::Bullet(_) => 3,
                Stuff::Charge(_) => 4,
                Stuff::Wood(_) => 5,
                Stuff::Mineral(_) => 6,
                Stuff::Tissue { .. } => 7,
            };
            (group, super::called(&stuff).to_string())
        }
    }
}

impl Pack {
    /// Returns the total discrete item count of `stuff` across all 30 slots.
    pub fn count_of(&self, stuff: &Stuff) -> u32 {
        self.grid
            .iter()
            .flatten()
            .filter_map(|e| match *e {
                SlotEntry::Stack {
                    stuff: s, count, ..
                } if s == *stuff => Some(u32::from(count)),
                _ => None,
            })
            .sum()
    }

    /// Returns how many additional items of `stuff` fit into partial or empty slots.
    pub fn room_for(&self, stuff: &Stuff) -> u32 {
        self.grid
            .iter()
            .map(|slot| match *slot {
                None => u32::from(STACK_MAX),
                Some(SlotEntry::Stack {
                    stuff: s, count, ..
                }) if s == *stuff => u32::from(STACK_MAX.saturating_sub(count)),
                _ => 0,
            })
            .sum()
    }

    /// Adds up to `count` items of `(stuff, material)`, filling partial stacks then pack grid then
    /// hotbar slots, returning how many fit.
    pub fn add_stack(&mut self, stuff: Stuff, material: Material, mut count: u16) -> u16 {
        self.ensure_grid();
        let wanted = count;
        for slot in self.grid.iter_mut().flatten() {
            if let SlotEntry::Stack {
                stuff: s,
                count: has,
                ..
            } = slot
                && *s == stuff
                && *has < STACK_MAX
            {
                let add = count.min(STACK_MAX - *has);
                *has += add;
                count -= add;
                if count == 0 {
                    break;
                }
            }
        }
        for idx in (BAR_SLOTS..TOTAL_SLOTS).chain(0..BAR_SLOTS) {
            if count == 0 {
                break;
            }
            if self.grid[idx].is_none() {
                let put = count.min(STACK_MAX);
                self.grid[idx] = Some(SlotEntry::Stack {
                    stuff,
                    material,
                    count: put,
                });
                count -= put;
            }
        }
        self.sync_views();
        wanted - count
    }

    /// Removes exactly `want` items of `stuff` from pack grid first then hotbar, returning false if short.
    pub fn take_count(&mut self, stuff: &Stuff, mut want: u16) -> bool {
        self.ensure_grid();
        if self.count_of(stuff) < u32::from(want) {
            return false;
        }
        for idx in (BAR_SLOTS..TOTAL_SLOTS).chain(0..BAR_SLOTS) {
            if want == 0 {
                break;
            }
            if let Some(SlotEntry::Stack {
                stuff: s, count, ..
            }) = &mut self.grid[idx]
                && *s == *stuff
            {
                let take = want.min(*count);
                *count -= take;
                want -= take;
                if *count == 0 {
                    self.grid[idx] = None;
                }
            }
        }
        self.sync_views();
        true
    }

    /// Keeps a newly crafted firearm in the first free hotbar or pack slot and returns its id.
    pub fn keep_gun(&mut self, kind: GunKind, barrel: Mineral) -> Option<u32> {
        self.ensure_grid();
        let slot = (0..TOTAL_SLOTS).find(|&i| self.grid[i].is_none())?;
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        self.guns.push(Gun::new(id, kind, barrel));
        self.grid[slot] = Some(SlotEntry::Gun(id));
        self.sync_views();
        Some(id)
    }

    /// Swaps or merges slot `from` into slot `to`, returning true when anything changed.
    pub fn swap_slots(&mut self, from: usize, to: usize) -> bool {
        self.ensure_grid();
        if from >= TOTAL_SLOTS || to >= TOTAL_SLOTS || from == to {
            return false;
        }
        if let (
            Some(SlotEntry::Stack {
                stuff: a,
                count: ca,
                ..
            }),
            Some(SlotEntry::Stack {
                stuff: b,
                material: mb,
                count: cb,
            }),
        ) = (self.grid[from], self.grid[to])
            && a == b
            && cb < STACK_MAX
        {
            let move_n = ca.min(STACK_MAX - cb);
            self.grid[to] = Some(SlotEntry::Stack {
                stuff: b,
                material: mb,
                count: cb + move_n,
            });
            let left = ca - move_n;
            self.grid[from] = (left > 0).then_some(SlotEntry::Stack {
                stuff: a,
                material: mb,
                count: left,
            });
            self.sync_views();
            return true;
        }
        self.grid.swap(from, to);
        self.sync_views();
        true
    }

    /// Takes half (rounded up) of slot `at` into a detached `SlotEntry`.
    pub fn split_slot(&mut self, at: usize) -> Option<SlotEntry> {
        self.ensure_grid();
        let entry = self.grid.get(at).copied().flatten()?;
        match entry {
            SlotEntry::Stack {
                stuff,
                material,
                count,
            } if count > 1 => {
                let half = count.div_ceil(2);
                self.grid[at] = Some(SlotEntry::Stack {
                    stuff,
                    material,
                    count: count - half,
                });
                self.sync_views();
                Some(SlotEntry::Stack {
                    stuff,
                    material,
                    count: half,
                })
            }
            other => {
                self.grid[at] = None;
                self.sync_views();
                Some(other)
            }
        }
    }

    /// Quick-moves slot `from` between the hotbar (`0..6`) and pack grid (`6..30`).
    pub fn quick_move(&mut self, from: usize) -> bool {
        self.ensure_grid();
        let Some(entry) = self.grid.get(from).copied().flatten() else {
            return false;
        };
        let range: Vec<usize> = if from < BAR_SLOTS {
            (BAR_SLOTS..TOTAL_SLOTS).collect()
        } else {
            (0..BAR_SLOTS).collect()
        };
        if let SlotEntry::Stack {
            stuff,
            material,
            mut count,
        } = entry
        {
            for &dst in &range {
                if let Some(SlotEntry::Stack {
                    stuff: s,
                    count: has,
                    ..
                }) = &mut self.grid[dst]
                    && *s == stuff
                    && *has < STACK_MAX
                {
                    let add = count.min(STACK_MAX - *has);
                    *has += add;
                    count -= add;
                    if count == 0 {
                        break;
                    }
                }
            }
            self.grid[from] = (count > 0).then_some(SlotEntry::Stack {
                stuff,
                material,
                count,
            });
            if count == 0 {
                self.sync_views();
                return true;
            }
        }
        let Some(&dst) = range.iter().find(|&&i| self.grid[i].is_none()) else {
            self.sync_views();
            return false;
        };
        self.grid.swap(from, dst);
        self.sync_views();
        true
    }

    /// Merges matching stacks in the 24-slot pack grid (`6..30`) and sorts them by item category.
    pub fn sort_grid(&mut self) {
        self.ensure_grid();
        let mut raw: Vec<SlotEntry> = self.grid[BAR_SLOTS..TOTAL_SLOTS]
            .iter_mut()
            .filter_map(Option::take)
            .collect();
        let mut merged: Vec<SlotEntry> = Vec::with_capacity(raw.len());
        for item in raw.drain(..) {
            match item {
                SlotEntry::Stack {
                    stuff,
                    material,
                    mut count,
                } => {
                    for existing in &mut merged {
                        if let SlotEntry::Stack {
                            stuff: s,
                            count: has,
                            ..
                        } = existing
                            && *s == stuff
                            && *has < STACK_MAX
                        {
                            let add = count.min(STACK_MAX - *has);
                            *has += add;
                            count -= add;
                        }
                    }
                    while count > 0 {
                        let put = count.min(STACK_MAX);
                        merged.push(SlotEntry::Stack {
                            stuff,
                            material,
                            count: put,
                        });
                        count -= put;
                    }
                }
                other => merged.push(other),
            }
        }
        merged.sort_by(|a, b| sort_key(self, a).cmp(&sort_key(self, b)));
        for (idx, item) in merged.into_iter().take(GRID_SLOTS).enumerate() {
            self.grid[BAR_SLOTS + idx] = Some(item);
        }
        self.sync_views();
    }

    /// Puts tool `id` into hotbar slot `slot`, keeping `grid` and `bar.slots` synchronized.
    pub fn put_bar(&mut self, slot: usize, id: u32) {
        self.ensure_grid();
        if slot >= BAR_SLOTS {
            return;
        }
        self.bar.put(slot, id);
        self.sync_bar_into_grid();
        self.sync_views();
    }
}
