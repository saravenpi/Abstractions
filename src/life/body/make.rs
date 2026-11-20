//! What every archetype builder works with: the kit, the traits from the rule table, the guild and the dice.

use crate::life::{Guild, Habitat};
use crate::universe::hash::Rng;

use super::kit::Kit;
use super::rules::Traits;

/// The state an archetype builder adds parts to.
#[derive(Debug, Clone)]
pub struct Make {
    /// The body so far.
    pub kit: Kit,
    /// What the rule table decided.
    pub t: Traits,
    /// How the species eats, which shapes the head.
    pub guild: Guild,
    /// Where it lives.
    pub habitat: Habitat,
    /// Seeded dice for the proportions.
    pub rng: Rng,
}

impl Make {
    /// Returns a factor drawn between `1 - spread` and `1 + spread`.
    pub fn vary(&mut self, spread: f32) -> f32 {
        self.rng
            .range(f64::from(1.0 - spread), f64::from(1.0 + spread)) as f32
    }

    /// Returns true with probability `odds`.
    pub fn chance(&mut self, odds: f32) -> bool {
        self.rng.unit() < f64::from(odds)
    }
}
