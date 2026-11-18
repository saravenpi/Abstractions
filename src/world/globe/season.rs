//! The season of a globe: how far round its orbit the planet is now, set from outside a few times a minute.

use std::sync::atomic::{AtomicU32, Ordering};

use super::Globe;

/// The season of the annual mean, when the tilt warms neither hemisphere: what the ground is shaped by.
pub const MEAN_SEASON: f32 = 0.25;

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

    /// Sets the current season, wrapped into 0 to 1. Only [`Globe::climate`] reads it: the ground, its
    /// biomes and its meshes follow the annual mean, so nothing has to be rebuilt.
    pub fn set_season(&self, season: f32) {
        let wrapped = season.rem_euclid(1.0);
        self.season.0.store(wrapped.to_bits(), Ordering::Relaxed);
    }
}
