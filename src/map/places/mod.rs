//! Pins: the places a player marks on a planet. One system for what were waypoints and
//! favourites.
//!
//! A pin has a name and a place. The player has at most [`MAX`] of them, saved with the world.
//! One may be the active pin, the one the compass leads to at any distance. The pin key drops a
//! pin at the feet, or removes the pin within [`NEAR`] metres of them; a right click on the
//! ground of the map drops one there. On a server the active pin goes out to the others and
//! theirs come back into [`Places::friends`].

mod keys;
mod name;
mod store;

pub use name::{NAME_CHARS, edit};

pub(super) use keys::{announce, arrive, drop_key, leading};
pub use keys::{press, reached};
pub(super) use name::name_keys;
pub(super) use store::{fill, load, plugin, store};

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::saves::Pin;
use crate::universe::BodyId;

/// Most pins a player keeps.
pub const MAX: usize = 8;

/// Metres within which the pin key removes a pin instead of dropping a new one.
pub const NEAR: f64 = 10.0;

/// Metres from the active pin at which the player has arrived.
pub const HERE: f64 = 12.0;

/// The pins of this player and the active pins of the others.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Places {
    /// The id of this player on the server, which picks its colour; 0 alone.
    pub me: u32,
    /// The pins of this player, in the order they were dropped.
    pub pins: Vec<Pin>,
    /// The index of the active pin.
    pub active: Option<usize>,
    /// The index of the pin whose card is shown. It is not saved.
    pub selected: Option<usize>,
    /// The active pin of each other player, by their id.
    pub friends: HashMap<u32, Pin>,
}

/// Returns `index` after the pin at `gone` was removed: nothing for that pin, one less past it.
fn shifted(index: Option<usize>, gone: usize) -> Option<usize> {
    index
        .filter(|at| *at != gone)
        .map(|at| at - usize::from(at > gone))
}

impl Places {
    /// Drops a pin at the direction `dir` of `body`, named with the next free number, and
    /// returns its index. Nothing happens when [`MAX`] pins are kept or `dir` is no direction.
    pub fn drop(&mut self, body: BodyId, dir: DVec3) -> Option<usize> {
        let dir = dir.try_normalize().filter(|_| self.pins.len() < MAX)?;
        let name = (1..)
            .map(|number| format!("Pin {number}"))
            .find(|name| self.pins.iter().all(|pin| pin.name != *name))?;
        self.pins.push(Pin {
            name,
            body,
            dir: dir.to_array(),
        });
        Some(self.pins.len() - 1)
    }

    /// Removes the pin at `index`, and returns it. The active and the selected pin follow.
    pub fn remove(&mut self, index: usize) -> Option<Pin> {
        let gone = (index < self.pins.len()).then(|| self.pins.remove(index))?;
        self.active = shifted(self.active, index);
        self.selected = shifted(self.selected, index);
        Some(gone)
    }

    /// Gives the pin at `index` the name `name`, unless the name is blank.
    pub fn rename(&mut self, index: usize, name: &str) {
        let name = name.trim();
        if let Some(pin) = self.pins.get_mut(index).filter(|_| !name.is_empty()) {
            pin.name = name.chars().take(NAME_CHARS).collect();
        }
    }

    /// Returns the active pin.
    pub fn active(&self) -> Option<&Pin> {
        self.active.and_then(|index| self.pins.get(index))
    }

    /// Makes the pin at `index` the active one, or none active. An index past the end is none.
    pub fn set_active(&mut self, index: Option<usize>) {
        self.active = index.filter(|index| *index < self.pins.len());
    }

    /// Returns the index of the pin of `body` in the direction `dir`.
    pub fn at(&self, body: BodyId, dir: DVec3) -> Option<usize> {
        let dir = dir.try_normalize()?;
        let same = |pin: &Pin| pin.body == body && DVec3::from_array(pin.dir).dot(dir) > 1.0 - 1e-9;
        self.pins.iter().position(same)
    }

    /// Returns the index of the pin of `body` within [`NEAR`] metres of the point `pos`, given
    /// in metres from the centre of the body, which is `radius` metres in radius.
    pub fn near(&self, body: BodyId, pos: DVec3, radius: f64) -> Option<usize> {
        let dir = pos.try_normalize()?;
        let close = |pin: &Pin| {
            pin.body == body && (DVec3::from_array(pin.dir) - dir).length() * radius <= NEAR
        };
        self.pins.iter().position(close)
    }
}

#[cfg(test)]
#[path = "../places_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "../places_end_tests.rs"]
mod tests_end;
#[cfg(test)]
#[path = "../places_store_tests.rs"]
mod tests_store;
