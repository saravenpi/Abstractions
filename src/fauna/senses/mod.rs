//! Senses: how each drawn animal notices the robots and hunters near it, and what it does about them.
//!
//! Every frame an animal's awareness meter (see [`meter`]) fills towards each threat it sees over the
//! ground, hears or smells downwind, and drains once it does not. The threat it is most aware of
//! sets its [`Stance`] by fight or flight (see [`stance`]); the walk turns its head to it, makes it
//! flee, stand or approach, and the hearing makes it call. Each herd has one lookout with keener
//! senses whose flight sets off the whole herd. A wounded animal limps to its den and bleeds.

mod blood;
mod cues;
mod feel;
mod health;
mod judge;
mod meter;
mod sense;
pub mod stance;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_blood;
#[cfg(test)]
mod tests_cost;
#[cfg(test)]
mod tests_crouch;
#[cfg(test)]
mod tests_stance;
pub mod tune;

use bevy::math::{DVec3, Vec3};
use bevy::prelude::*;

pub use blood::{Blood, Drip};
pub use health::Health;
pub use meter::Meter;
pub(super) use sense::sense;
pub use stance::Stance;

use super::attack::Broken;
use crate::life::Species;

/// The thing an animal is most aware of.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Focus {
    /// Its place in the animal's frame.
    pub at: Vec3,
    /// How aware of it the animal is, 0 to 1.
    pub aware: f32,
}

/// What one animal knows of the threats around it and of itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wits {
    /// Its meters, one per threat.
    pub meter: Meter,
    /// What it does about them.
    pub stance: Stance,
    /// The threat it is most aware of.
    pub focus: Option<Focus>,
    /// Its health.
    pub health: Health,
    /// Unit direction of the den a wounded animal heads for.
    pub den: Option<DVec3>,
    /// Planet position of its last drop of blood.
    pub bled: Option<DVec3>,
    /// The parts of its body that are broken.
    pub broken: Broken,
}

impl Wits {
    /// Returns the wits of a fresh, unhurt animal of `species`.
    pub fn of(species: &Species) -> Self {
        Self {
            meter: Meter::default(),
            stance: Stance::Calm,
            focus: None,
            health: Health::of(species.history.mass),
            den: None,
            bled: None,
            broken: Broken::default(),
        }
    }
}

/// Registers the resources of the senses.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Blood>();
}
