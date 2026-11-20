//! The pack: the tools the robot made and the materials it carries, limited by mass, not by slots.
//!
//! Tools are kept as recipes, materials stack by where they came from (species, tissue, grade and
//! treatments, or the stone). [`Burden`] says what the load weighs on this world and how much it
//! slows the walk. The pack is saved in the world file.

mod bar;
mod item;
mod lend;
mod load;
mod make;
mod pick;
mod save;
mod slots;
mod tune;
mod words;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_lend;
#[cfg(test)]
mod tests_pick;

pub use bar::{Bar, SLOTS};
pub use item::Item;
pub use lend::Lent;
pub use load::{Burden, capacity, pace};
pub use make::cost;
pub(crate) use pick::gather;
pub use pick::stones;
pub(crate) use save::{load, store, weigh};
pub use words::{called, noun};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::craft::build::{Recipe, Stock};
use crate::craft::matter::{Material, Stuff};
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

/// Everything the robot carries.
#[derive(Resource, Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pack {
    /// The made tools.
    pub items: Vec<Item>,
    /// The materials.
    pub stacks: Vec<Stack>,
    /// The hotbar.
    pub bar: Bar,
    /// The id the next made tool gets.
    pub next: u32,
}

impl Pack {
    /// Returns the mass of everything carried, in kilograms.
    pub fn mass(&self) -> f32 {
        let tools: f32 = self.items.iter().map(Item::mass).sum();
        tools + self.stacks.iter().map(|s| s.mass).sum::<f32>()
    }

    /// Adds up to `stack.mass` of its material, no more than `room` kilograms, onto the stack of the
    /// same stuff or a new one, and returns the mass added.
    pub fn stow(&mut self, stack: Stack, room: f32) -> f32 {
        let kept = stack.mass.min(room.max(0.0));
        if kept < CRUMB {
            return 0.0;
        }
        match self.stacks.iter_mut().find(|s| s.stuff == stack.stuff) {
            Some(heap) => heap.mass += kept,
            None => self.stacks.push(Stack {
                mass: kept,
                ..stack
            }),
        }
        kept
    }

    /// Takes `mass` kilograms of `stuff`, and returns false, taking nothing, when there is less.
    pub fn take(&mut self, stuff: &Stuff, mass: f32) -> bool {
        let Some(at) = self.stacks.iter().position(|s| s.stuff == *stuff) else {
            return false;
        };
        if self.stacks[at].mass + CRUMB < mass {
            return false;
        }
        self.stacks[at].mass -= mass;
        if self.stacks[at].mass < CRUMB {
            self.stacks.remove(at);
        }
        true
    }

    /// Keeps a made tool of `recipe` and `materials`, puts it in a free hotbar slot and returns its id.
    pub fn keep(&mut self, recipe: Recipe, materials: Vec<Material>) -> u32 {
        let id = self.next;
        self.next = self.next.wrapping_add(1);
        self.items.push(Item {
            id,
            recipe,
            materials,
        });
        self.bar.place(id);
        id
    }

    /// Returns the tool with `id`.
    pub fn item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Returns the held tool.
    pub fn held(&self) -> Option<&Item> {
        self.bar.held().and_then(|id| self.item(id))
    }

    /// Returns the materials as the solver reads them.
    pub fn stock(&self) -> Vec<Stock> {
        let stock = |s: &Stack| Stock {
            stuff: s.stuff,
            material: s.material,
            mass: s.mass,
        };
        self.stacks.iter().map(stock).collect()
    }
}
