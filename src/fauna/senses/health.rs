//! Health: how much damage an animal takes before it dies, from its mass, and the limp of a wounded
//! one.

use super::tune::{HEALTH, LIMP, WOUNDED};
use crate::life::Species;

/// The health of one animal, in joules of damage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    /// Health when unhurt.
    pub max: f32,
    /// Health left.
    pub now: f32,
}

impl Health {
    /// Returns the full health of an animal of `mass` kilograms: [`HEALTH`] joules per kilogram to
    /// the power 0.75.
    pub fn of(mass: f32) -> Self {
        let max = HEALTH * mass.max(0.01).powf(0.75);
        Self { max, now: max }
    }

    /// Returns the full health of an animal of `species`: bosses have 2.5 times normal health.
    pub fn of_species(species: &Species) -> Self {
        let mut health = Self::of(species.history.mass);
        if species.boss {
            health.max *= 2.5;
            health.now = health.max;
        }
        health
    }

    /// Returns the share of health left, 0 to 1.
    pub fn share(&self) -> f32 {
        (self.now / self.max.max(1e-6)).clamp(0.0, 1.0)
    }

    /// Returns true when under [`WOUNDED`] of its health is left.
    pub fn wounded(&self) -> bool {
        self.share() < WOUNDED
    }

    /// Returns the share of its speed the animal goes at: [`LIMP`] when wounded.
    pub fn pace(&self) -> f32 {
        if self.wounded() { LIMP } else { 1.0 }
    }

    /// Takes `joules` of damage and returns true when it kills. The caller records the death in the
    /// kill ledger, `crate::life::herd::Ledger`.
    pub fn hurt(&mut self, joules: f32) -> bool {
        self.now = (self.now - joules.max(0.0)).max(0.0);
        self.now <= 0.0
    }
}
