//! Build: a tool as parts on one axis through the hand, and the physics that follows from them.
//!
//! A build is a grip, a shaft, a head and an optional counterweight, each a mass at a distance from
//! the hand. From those follow the moment of inertia, the swing and thrust speeds, the hold torque on
//! this world, the damage against a target's layers and the wear of each joint. [`solve`] answers an
//! intent with three proposals from what a player carries; [`Recipe`] stores an item in a few bytes.

mod blow;
mod hit;
mod inertia;
mod part;
mod recipe;
mod solve;
mod swing;
mod throw;
mod tune;
mod wear;
mod why;

#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests_hit;
#[cfg(test)]
mod tests_inertia;
#[cfg(test)]
mod tests_recipe;
#[cfg(test)]
mod tests_solve;
#[cfg(test)]
mod tests_wear;

pub use blow::blow;
pub use hit::{Blow, Layer, Mode, Strike, reference, strike};
pub use inertia::{Piece, balance, inertia, mass, moment};
pub use part::{Build, Form, Head, Shaft, Weight};
pub use recipe::{Design, Recipe};
pub use solve::{Proposal, Proposals, Stock, solve};
pub use swing::{Swing, swing, tip_speed};
pub use throw::{Throw, throw};
pub use wear::{Joint, Wear, lives};
pub use why::{Why, explain};

use serde::{Deserialize, Serialize};

use crate::universe::PlanetSpec;

/// What the player asks a tool to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    /// Slash with an edge.
    Cut,
    /// Thrust with a point.
    Pierce,
    /// Swing a heavy head.
    Crush,
    /// Throw a spear.
    Throw,
}

impl Intent {
    /// Returns the head form the intent needs.
    pub fn form(self) -> Form {
        match self {
            Intent::Cut => Form::Edge,
            Intent::Pierce | Intent::Throw => Form::Point,
            Intent::Crush => Form::Mass,
        }
    }
}

/// How many hands hold the grip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hands {
    /// One hand.
    One,
    /// Both hands.
    Two,
}

impl Hands {
    /// Returns the torque the grip holds level, in newton metres.
    pub fn budget(self) -> f32 {
        match self {
            Hands::One => tune::ONE_HAND,
            Hands::Two => tune::TWO_HANDS,
        }
    }
}

/// The world a tool is used on, as the physics reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    /// Surface gravity in metres per second squared.
    pub gravity: f32,
    /// Air density at the surface in kilograms per cubic metre.
    pub air: f32,
}

impl Place {
    /// Returns the place on the surface of `planet`. Air density follows the ideal gas law from the
    /// home air: ρ = 1.225 kg/m³ × pressure × 288 K / temperature.
    pub fn of(planet: &PlanetSpec) -> Self {
        let record = &planet.record;
        Self {
            gravity: planet.gravity as f32,
            air: (tune::SEA_AIR * record.pressure * 288.0 / record.temperature.max(1.0)) as f32,
        }
    }
}
