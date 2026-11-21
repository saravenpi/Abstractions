//! The season of a globe: how far round its orbit the planet is now, set from outside a few times a minute.

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::math::DVec3;

use super::Globe;
use crate::climate::{self, Climate, Spot};

pub use crate::climate::MEAN_SEASON;

/// The current season, from 0 to 1, stored so that a shared globe can be told it without a lock.
#[derive(Debug)]
pub struct Season(AtomicU32);

impl Default for Season {
    fn default() -> Self {
        Self(AtomicU32::new(MEAN_SEASON.to_bits()))
    }
}

impl Clone for Season {
    fn clone(&self) -> Self {
        Self(AtomicU32::new(self.0.load(Ordering::Relaxed)))
    }
}

impl Globe {
    /// Returns the current season, from 0 to 1 round the orbit; 0.25 until it is set.
    pub fn season(&self) -> f32 {
        f32::from_bits(self.season.0.load(Ordering::Relaxed))
    }

    /// Sets the current season, wrapped into 0 to 1. Only [`Globe::air`] reads it: the ground, its biomes
    /// and its meshes follow the annual mean, so nothing has to be rebuilt.
    pub fn set_season(&self, season: f32) {
        let wrapped = season.rem_euclid(1.0);
        self.season.0.store(wrapped.to_bits(), Ordering::Relaxed);
    }

    /// Returns the climate on the ground in unit direction `dir` at the current season, which the tests
    /// compare with the climate of a given season.
    #[cfg(test)]
    pub fn climate(&self, dir: DVec3) -> Climate {
        self.climate_in(dir, self.season())
    }

    /// Returns the climate of the moment in unit direction `dir`, `altitude` metres above the liquid
    /// surface (above the datum on a dry world) and never under the ground there: the air a camera or a
    /// player is in. It reads the place the biomes are picked from and the current season.
    pub fn air(&self, dir: DVec3, altitude: f32) -> Climate {
        let ground = self.spot(dir);
        let spot = Spot {
            altitude: altitude.max(ground.altitude),
            ..ground
        };
        climate::at(&self.setting, &spot, self.season())
    }
}
