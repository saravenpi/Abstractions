//! Blood: the drops a wounded animal leaves every [`BLOOD_STEP`] metres, drawn with the signs of life
//! and faded over the footprint's [`TRAIL`].
//!
//! Wounds are local to each player until the server holds them, so blood is not a pure function of
//! the herds like the other signs: the runtime keeps the drops it saw fall.

use std::collections::VecDeque;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{BLOOD_MOST, BLOOD_STEP};
use crate::fauna::herds::herd_arc;
use crate::life::signs::tune::TRAIL;
use crate::life::signs::{Mark, Sign};

/// One drop of blood.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drip {
    /// Unit direction where it fell.
    pub at: DVec3,
    /// Unit tangent at `at` the animal went along.
    pub facing: DVec3,
    /// Index of the species that bled.
    pub species: u16,
    /// Length in metres.
    pub size: f32,
    /// Universe time it fell, in seconds.
    pub time: f64,
}

/// The drops of blood fallen on the planet under the anchor, oldest first.
#[derive(Resource, Debug, Default)]
pub struct Blood {
    /// The drops, at most [`BLOOD_MOST`].
    drops: VecDeque<Drip>,
}

impl Blood {
    /// Adds `drip` when it lies [`BLOOD_STEP`] metres or more from `last`, the planet position of
    /// the last drop of that animal on a planet of `radius` metres, and returns the planet position
    /// of the last drop now.
    pub fn bleed(&mut self, drip: Drip, last: Option<DVec3>, radius: f64) -> DVec3 {
        let here = drip.at * radius;
        if last.is_some_and(|l| l.distance(here) < BLOOD_STEP) {
            return last.unwrap_or(here);
        }
        while self.drops.len() >= BLOOD_MOST {
            self.drops.pop_front();
        }
        self.drops.push_back(drip);
        here
    }

    /// Adds to `out` the drops younger than [`TRAIL`] at universe time `now` within `reach` metres
    /// along the ground of unit direction `at` on a planet of `radius` metres, and forgets older ones.
    pub fn lay(
        &mut self,
        (at, reach): (DVec3, f64),
        (now, radius): (f64, f64),
        out: &mut Vec<Sign>,
    ) {
        while self.drops.front().is_some_and(|d| now - d.time > TRAIL) {
            self.drops.pop_front();
        }
        for drop in self.drops.iter().filter(|d| d.time <= now) {
            if herd_arc(drop.at, at) * radius > reach {
                continue;
            }
            out.push(Sign {
                mark: Mark::Blood,
                species: drop.species,
                at: drop.at,
                facing: drop.facing,
                size: drop.size,
                fresh: (1.0 - (now - drop.time) / TRAIL) as f32,
            });
        }
    }

    /// Forgets every drop.
    pub fn clear(&mut self) {
        self.drops.clear();
    }

    /// Returns how many drops are kept.
    pub fn len(&self) -> usize {
        self.drops.len()
    }

    /// Returns true when no drop is kept.
    pub fn is_empty(&self) -> bool {
        self.drops.is_empty()
    }
}
