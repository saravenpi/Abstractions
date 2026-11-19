//! The ground uncovered on each body: a 512 by 256 grid in longitude and latitude, one bit a
//! cell, row by row from the south, the layout of the save and of the wire.
//!
//! Walking and flying low uncover the cells around the player. Cells uncovered here wait in a
//! list for the network to send them; cells that come from the server are only marked.

pub mod grid;
mod ground;
mod reveal;
mod share;

pub(super) use reveal::plugin;
pub use reveal::{REACH_CAP, REACH_FOOT, reach};

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;

use self::ground::Ground;
use crate::saves::WorldSave;
use crate::universe::{BodyId, Universe};

/// The ground uncovered on every body.
#[derive(Resource, Debug, Clone, Default)]
pub struct Explored {
    /// The bits of each body uncovered so far.
    bodies: HashMap<BodyId, Ground>,
}

impl Explored {
    /// Returns the ground a world file records. A file without any is uncovered around the
    /// place the player last stood.
    pub fn of(save: &WorldSave) -> Self {
        let mut explored = Self::default();
        for saved in &save.explored {
            explored.merge_words(saved.body, &saved.words);
        }
        let place = save.place.filter(|_| save.explored.is_empty());
        if let Some(place) = place {
            let dir = DVec3::from(place.dir).try_normalize().unwrap_or(DVec3::Y);
            let radius = Universe::new(save.seed).planet(place.body).radius;
            explored.reveal(place.body, dir, REACH_CAP / radius.max(1.0));
        }
        explored
    }

    /// Returns the words of the bitset of `body`, 2048 of them, or nothing before any cell of it
    /// was uncovered.
    pub fn words(&self, body: BodyId) -> Option<&[u64]> {
        self.bodies.get(&body).map(|ground| ground.words.as_slice())
    }

    /// Returns a number that grows every time a cell of `body` is uncovered, 0 before any was.
    pub fn version(&self, body: BodyId) -> u64 {
        self.bodies.get(&body).map_or(0, |ground| ground.version)
    }

    /// Returns every body with the words of its bitset.
    pub fn bodies(&self) -> impl Iterator<Item = (BodyId, &[u64])> {
        self.bodies
            .iter()
            .map(|(body, ground)| (*body, ground.words.as_slice()))
    }

    /// Marks cells that another player uncovered. They are not sent back to the server.
    pub fn reveal_cells(&mut self, body: BodyId, cells: &[u32]) {
        let ground = self.bodies.entry(body).or_default();
        for &cell in cells {
            ground.set(cell);
        }
    }

    /// Marks every cell set in `words`, a whole bitset as the server or a save holds it.
    pub fn merge_words(&mut self, body: BodyId, words: &[u64]) {
        self.bodies.entry(body).or_default().merge(words);
    }

    /// Returns the cells of `body` uncovered here since the last call, for the network to send.
    pub fn take_new(&mut self, body: BodyId) -> Vec<u32> {
        self.bodies
            .get_mut(&body)
            .map(|ground| std::mem::take(&mut ground.new))
            .unwrap_or_default()
    }

    /// Uncovers the cells of `body` within `angle` radians of unit direction `dir`, and the cell
    /// under it, keeping the new ones for the network.
    pub fn reveal(&mut self, body: BodyId, dir: DVec3, angle: f64) {
        let ground = self.bodies.entry(body).or_default();
        grid::disc(dir, angle, |cell| {
            if ground.set(cell) {
                ground.new.push(cell);
            }
        });
    }
}

#[cfg(test)]
mod tests;
