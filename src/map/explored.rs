//! The ground uncovered on each body: leaves of a quadtree on its cube-sphere, about 500 m
//! across, kept as runs of keys, the form of the save and of the wire.
//!
//! Walking and flying low uncover the leaves around the player. Leaves uncovered here wait in a
//! set for the network to send them; leaves that come from the server are only marked.

pub mod disc;
pub mod grid;
mod ground;
mod morton;
mod remote;
mod reveal;
mod share;

pub(super) use reveal::plugin;
pub use reveal::{REACH_CAP, REACH_FOOT, reach};

use std::collections::HashMap;

use abstractions_net::leaves::Leaves;
use bevy::math::DVec3;
use bevy::prelude::*;

use self::ground::Ground;
use crate::saves::WorldSave;
use crate::universe::{BodyId, Universe};

/// The ground uncovered on every body.
#[derive(Resource, Debug, Clone, Default)]
pub struct Explored {
    /// The leaves of each body uncovered so far.
    bodies: HashMap<BodyId, Ground>,
}

impl Explored {
    /// Returns the ground a world file records. A file without any is uncovered around the
    /// place the player last stood.
    pub fn of(save: &WorldSave) -> Self {
        let mut explored = Self::default();
        for saved in &save.explored {
            explored.merge_runs(saved.body, &saved.runs);
        }
        let place = save.place.filter(|_| save.explored.is_empty());
        if let Some(place) = place {
            let dir = DVec3::from(place.dir).try_normalize().unwrap_or(DVec3::Y);
            let radius = Universe::new(save.seed).planet(place.body).radius;
            explored.reveal(place.body, dir, REACH_CAP / radius.max(1.0), radius);
        }
        explored
    }

    /// Returns the leaves of `body`, or nothing before any of it was uncovered.
    pub fn leaves(&self, body: BodyId) -> Option<&Leaves> {
        self.bodies.get(&body).map(|ground| &ground.leaves)
    }

    /// Returns a number that grows every time a leaf of `body` is uncovered, 0 before any was.
    pub fn version(&self, body: BodyId) -> u64 {
        self.bodies.get(&body).map_or(0, |ground| ground.version)
    }

    /// Returns every body with its leaves.
    pub fn bodies(&self) -> impl Iterator<Item = (BodyId, &Leaves)> {
        self.bodies
            .iter()
            .map(|(body, ground)| (*body, &ground.leaves))
    }

    /// Uncovers the leaves of `body`, a body of `radius` metres, within `angle` radians of unit
    /// direction `dir`, and the leaf under it, keeping the new ones for the network.
    pub fn reveal(&mut self, body: BodyId, dir: DVec3, angle: f64, radius: f64) {
        let Some(ground) = self.ground(body) else {
            return;
        };
        disc::disc(dir, angle, grid::depth_for(radius), |leaf| {
            ground.set(leaf);
        });
    }
}

#[cfg(test)]
mod share_state_tests;
#[cfg(test)]
mod tests;
