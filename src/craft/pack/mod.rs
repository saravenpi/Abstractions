//! The pack: a 24-slot grid plus 6 hotbar slots (stacks up to 64), tools, and firearms.

mod bar;
mod grid_ops;
mod item;
mod kit;
mod lend;
mod load;
mod make;
mod notice;
mod pick;
mod save;
mod slot;
mod slots;
pub mod tune;
mod words;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_kit;
#[cfg(test)]
mod tests_knock;
#[cfg(test)]
mod tests_lend;
#[cfg(test)]
mod tests_pick;

pub use bar::{Bar, SLOTS};
pub use item::Item;
pub(crate) use kit::{kit, unkitted};
pub use lend::{Lent, Out};
pub use load::{Burden, capacity, pace};
pub use make::cost;
pub use notice::Notice;
pub(crate) use notice::{fade, fading};
pub(crate) use pick::chop::handle_chop;
pub(crate) use pick::dig::handle_dig;
pub use pick::{Forage, Foraged, HarvestTarget, Knock, Knocked};
pub(crate) use pick::{gather, split};
pub(crate) use save::{load, store, weigh};
pub use slot::{BAR_SLOTS, GRID_SLOTS, STACK_MAX, SlotEntry, TOTAL_SLOTS};
pub use tune::PICK_KEY;
pub use words::{called, mineral_name, noun};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::craft::armor::Armor;
use crate::craft::build::{Recipe, Stock};
use crate::craft::gun::Gun;
use crate::craft::matter::{Material, Stuff, WoodKind};
use slot::units_from_mass;
use tune::CRUMB;

/// A heap of one material.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    /// Where it came from.
    pub stuff: Stuff,
    /// Its numbers, treatments included, as read where it was taken.
    pub material: Material,
    /// Mass carried, in kilograms.
    pub mass: f32,
}

/// Everything the robot carries: 6 hotbar slots and 24 pack slots plus tool/firearm storage.
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pack {
    /// The made tools.
    pub items: Vec<Item>,
    /// Crafted firearms referenced by id from `grid`.
    pub guns: Vec<Gun>,
    /// Crafted armor pieces referenced by id from `grid` or `equipped`.
    #[serde(default)]
    pub armors: Vec<Armor>,
    /// Equipped armor ids indexed by [`crate::craft::armor::ArmorSlot`].
    #[serde(default, with = "slot::serde_equipped")]
    pub equipped: [Option<u32>; 4],
    /// Compatibility grouped view of materials in `grid`.
    pub stacks: Vec<Stack>,
    /// The hotbar selection (`held` index and tool-id slots synced from `grid[0..6]`).
    pub bar: Bar,
    /// Sparse-serialized 30-slot grid (`0..6` hotbar, `6..30` 6x4 pack grid).
    #[serde(with = "slot::serde_grid")]
    pub grid: [Option<SlotEntry>; TOTAL_SLOTS],
    /// The id the next made tool or gun gets.
    pub next: u32,
    /// The tools that are out in the world: thrown and not picked up yet.
    pub out: Vec<Out>,
    /// True once the pack was given the starter kit.
    pub kitted: bool,
}

impl Default for Pack {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            guns: Vec::new(),
            armors: Vec::new(),
            equipped: [None; 4],
            stacks: Vec::new(),
            bar: Bar::default(),
            grid: [None; TOTAL_SLOTS],
            next: 0,
            out: Vec::new(),
            kitted: false,
        }
    }
}

impl Pack {
    /// Returns the total mass carried in kilograms, summing tools, guns, armor, and discrete stacks.
    pub fn mass(&self) -> f32 {
        let tools: f32 = self.items.iter().map(Item::mass).sum();
        let guns: f32 = self.guns.iter().map(Gun::mass).sum();
        let armors: f32 = self.armors.iter().map(|a| a.slot.mass_kg()).sum();
        tools + guns + armors + self.stacks.iter().map(|s| s.mass).sum::<f32>()
    }

    /// Adds up to `stack.mass` kilograms within `room` into discrete grid slots, returning added kg.
    pub fn stow(&mut self, stack: Stack, room: f32) -> f32 {
        self.ensure_grid();
        let kept = stack.mass.min(room.max(0.0));
        if kept < CRUMB {
            return 0.0;
        }
        let unit = stack.stuff.unit_mass().max(1e-4);
        let units = if room >= 1.0e9 {
            units_from_mass(&stack.stuff, kept)
        } else if kept < unit && room >= stack.mass {
            1
        } else {
            ((kept + CRUMB) / unit).floor() as u16
        };
        if units == 0 {
            return 0.0;
        }
        let added = self.add_stack(stack.stuff, stack.material, units);
        f32::from(added) * unit
    }

    /// Takes `mass` kilograms of `stuff` from discrete grid slots, returning false when short.
    pub fn take(&mut self, stuff: &Stuff, mass: f32) -> bool {
        self.ensure_grid();
        if mass < CRUMB {
            return true;
        }
        let unit = stuff.unit_mass().max(1e-4);
        let want = ((mass - CRUMB) / unit).ceil().max(1.0) as u16;
        self.take_count(stuff, want)
    }

    /// Keeps a made tool in the first free hotbar or pack slot and returns its id.
    pub fn keep(&mut self, recipe: Recipe, materials: Vec<Material>) -> u32 {
        self.ensure_grid();
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        self.items.push(Item {
            id,
            recipe,
            materials,
        });
        if let Some(slot) = (0..TOTAL_SLOTS).find(|&i| self.grid[i].is_none()) {
            self.grid[slot] = Some(SlotEntry::Tool(id));
        }
        self.sync_views();
        id
    }

    /// Returns the tool with `id`.
    pub fn item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Returns the firearm with `id`.
    pub fn gun(&self, id: u32) -> Option<&Gun> {
        self.guns.iter().find(|gun| gun.id == id)
    }

    /// Returns the armor piece with `id`.
    pub fn armor(&self, id: u32) -> Option<&Armor> {
        self.armors.iter().find(|armor| armor.id == id)
    }

    /// Returns a mutable reference to the firearm with `id`.
    pub fn gun_mut(&mut self, id: u32) -> Option<&mut Gun> {
        self.guns.iter_mut().find(|gun| gun.id == id)
    }

    /// Returns the entry currently selected in the hotbar (`0..6`).
    pub fn held_entry(&self) -> Option<&SlotEntry> {
        self.grid.get(self.bar.held).and_then(|s| s.as_ref())
    }

    /// Returns the held tool, if the selected hotbar slot holds a crafted tool.
    pub fn held(&self) -> Option<&Item> {
        match self.held_entry()? {
            SlotEntry::Tool(id) => self.item(*id),
            _ => None,
        }
    }

    /// Returns the held firearm, if the selected hotbar slot holds a gun.
    pub fn held_gun(&self) -> Option<&Gun> {
        match self.held_entry()? {
            SlotEntry::Gun(id) => self.gun(*id),
            _ => None,
        }
    }

    /// Returns a mutable reference to the held firearm, if the selected hotbar slot holds a gun.
    pub fn held_gun_mut(&mut self) -> Option<&mut Gun> {
        let &SlotEntry::Gun(id) = self.held_entry()? else {
            return None;
        };
        self.gun_mut(id)
    }

    /// Returns the materials usable by the procedural tool solver.
    pub fn stock(&self) -> Vec<Stock> {
        let usable = |s: &Stack| {
            matches!(
                s.stuff,
                Stuff::Tissue { .. }
                    | Stuff::Mineral(_)
                    | Stuff::Wood(WoodKind::Log | WoodKind::Plank | WoodKind::Stick)
            )
        };
        self.stacks
            .iter()
            .filter(|s| usable(s))
            .map(|s| Stock {
                stuff: s.stuff,
                material: s.material,
                mass: s.mass,
            })
            .collect()
    }
}
