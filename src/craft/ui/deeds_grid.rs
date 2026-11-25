//! Pointer grid interactions and Crafting Workshop deeds separated to keep `deeds.rs` compact.

use bevy::prelude::*;

use super::bench::Pane;
use super::catalog_ui::filtered_recipes;
use super::deeds::Deed;
use crate::craft::build::catalog::Category;
use crate::craft::build::catalog_craft::{craft_blueprint, max_craftable};
use crate::craft::ground::{Thing, shed_slot};
use crate::craft::matter::{Mineral, Stuff};
use crate::craft::pack::{BAR_SLOTS, SlotEntry, TOTAL_SLOTS};

impl Deed<'_> {
    /// Handles left-clicking a slot (`0..30`): Shift quick-moves, else selects or swaps/merges.
    pub fn click_grid_slot(&mut self, at: usize) {
        self.bench.pane = Pane::Pack;
        let shift = self
            .keys
            .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if shift {
            self.pack.quick_move(at);
            self.bench.picked_slot = None;
            self.bench.selected_slot = Some(at);
            return;
        }
        if let Some(from) = self.bench.picked_slot.take() {
            if from != at {
                self.pack.swap_slots(from, at);
            }
            self.bench.selected_slot = Some(at);
            return;
        }
        self.bench.selected_slot = Some(at);
        if self.pack.grid.get(at).is_some_and(Option::is_some) {
            self.bench.picked_slot = Some(at);
        }
    }

    /// Right-click splits a stack in slot `at` evenly into the first free slot.
    pub fn split_grid_slot(&mut self, at: usize) {
        self.bench.picked_slot = None;
        self.bench.selected_slot = Some(at);
        let Some(split) = self.pack.split_slot(at) else {
            return;
        };
        let free = (BAR_SLOTS..TOTAL_SLOTS)
            .chain(0..BAR_SLOTS)
            .find(|&i| i != at && self.pack.grid[i].is_none());
        if let Some(dst) = free {
            self.pack.grid[dst] = Some(split);
            self.pack.sync_views();
        } else {
            if let SlotEntry::Stack {
                stuff,
                material,
                count,
            } = split
            {
                self.pack.add_stack(stuff, material, count);
            } else {
                self.pack.grid[at] = Some(split);
                self.pack.sync_views();
            }
        }
    }

    /// Sorts and compacts all stacks inside the 24-slot pack grid.
    pub fn sort_pack(&mut self) {
        self.pack.sort_grid();
        self.bench.picked_slot = None;
        self.bench.note = "Sorted pack slots.".to_string();
    }

    /// Drops the currently selected grid slot (or falls back to row cursor).
    pub(super) fn drop_grid_or_row(&mut self, half: bool) -> bool {
        let Some(idx) = self.bench.selected_slot else {
            return false;
        };
        if self.pack.grid.get(idx).is_none_or(Option::is_none) {
            return false;
        }
        if half {
            if let Some(split) = self.pack.split_slot(idx) {
                if let SlotEntry::Stack {
                    stuff,
                    material,
                    count,
                } = split
                {
                    let mass = f32::from(count) * stuff.unit_mass();
                    self.shed.0.push(Thing::Heap(crate::craft::pack::Stack {
                        stuff,
                        material,
                        mass,
                    }));
                } else {
                    self.pack.grid[idx] = Some(split);
                    self.pack.sync_views();
                }
            }
        } else {
            shed_slot(&mut self.pack, &mut self.shed, idx);
        }
        self.bench.picked_slot = None;
        true
    }

    /// Crafts `times` batches (`0` means Craft Max) of the focused catalog recipe.
    pub fn craft_batch(&mut self, times: u16) {
        let recipes = filtered_recipes(self.bench.category, &self.pack, self.bench.bullet_mineral);
        let Some(bp) = recipes.get(self.bench.recipe_idx.min(recipes.len().saturating_sub(1)))
        else {
            self.bench.note = "No recipe selected.".to_string();
            return;
        };
        let n = if times == 0 {
            max_craftable(bp, &self.pack, self.bench.bullet_mineral).max(1)
        } else {
            times
        };
        self.bench.note = match craft_blueprint(bp, &mut self.pack, self.bench.bullet_mineral, n) {
            Ok(msg) => msg,
            Err(err) => err.to_string(),
        };
    }

    /// Cycles [`Bench::bullet_mineral`] through carried minerals first, falling back to [`Mineral::ALL`].
    pub fn cycle_mineral(&mut self) {
        let carried: Vec<Mineral> = Mineral::ALL
            .into_iter()
            .filter(|m| self.pack.count_of(&Stuff::Mineral(*m)) > 0)
            .collect();
        let pool = if carried.is_empty() {
            Mineral::ALL.as_slice()
        } else {
            carried.as_slice()
        };
        let pos = pool
            .iter()
            .position(|m| *m == self.bench.bullet_mineral)
            .map_or(0, |p| (p + 1) % pool.len());
        self.bench.bullet_mineral = pool[pos];
        let name = crate::craft::pack::called(&Stuff::Mineral(self.bench.bullet_mineral));
        self.bench.note = format!("Bullet ore: {name}.");
    }

    /// Handles category tab switching inside the Workshop.
    pub fn pick_category(&mut self, cat: Category) {
        self.bench.category = cat;
        self.bench.recipe_idx = 0;
        self.bench.note.clear();
    }
}
