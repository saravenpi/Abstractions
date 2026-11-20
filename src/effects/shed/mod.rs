//! What the plants around the camera shed: the crowns of the canopy within reach of the particle box,
//! nearest first, each with a leaf and a petal material in the colours of its species.

mod gather;

use bevy::prelude::*;

use super::kind::Kind;
use super::pool::{ABOVE, BELOW, HALF_WIDTH};
use super::rate::{self, Air};
use super::rng::Rng;
use crate::world::canopy::Shed;

pub use gather::gather;

/// Most plants read at once: one leaf and one petal material each.
pub const SLOTS: usize = 8;

/// Metres past the particle box within which a chunk may hold a crown that reaches into it.
pub(super) const MARGIN: f32 = 12.0;

/// One plant within reach of the particle box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Source {
    /// Where its stem meets the ground, in the local frame of the eye: `y` is up.
    pub at: Vec3,
    /// Height of the plant in metres.
    pub height: f32,
    /// Radius of its crown in metres.
    pub reach: f32,
    /// What falls from it.
    pub shed: Shed,
    /// Leaves per second it loses right now.
    pub leaf: f32,
    /// Petals per second it loses right now.
    pub petal: f32,
    /// Pollen specks per second it lets go right now.
    pub pollen: f32,
}

/// The plants within reach of the particle box, nearest first, at most [`SLOTS`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shedding {
    /// The plants.
    pub sources: Vec<Source>,
}

impl Shedding {
    /// Sets what each plant sheds per second in `air`.
    pub fn weigh(&mut self, air: &Air) {
        for source in &mut self.sources {
            source.leaf = rate::leaves(&source.shed, air);
            source.petal = rate::petals(&source.shed, air);
            source.pollen = rate::pollen(&source.shed, air);
        }
    }

    /// Returns the leaves per second all the plants within reach lose.
    pub fn leaves(&self) -> f32 {
        self.sources.iter().map(|s| s.leaf).sum()
    }

    /// Returns the petals per second all the plants within reach lose.
    pub fn petals(&self) -> f32 {
        self.sources.iter().map(|s| s.petal).sum()
    }

    /// Returns the pollen specks per second all the plants within reach let go.
    pub fn pollen(&self) -> f32 {
        self.sources.iter().map(|s| s.pollen).sum()
    }

    /// Returns the index of the plant a particle of `kind` comes from, drawn by `roll` in `[0, 1)`
    /// among the plants that shed it, or nothing when none does.
    pub fn choose(&self, kind: Kind, roll: f32) -> Option<usize> {
        let weight = |s: &Source| match kind {
            Kind::Leaf => s.leaf,
            Kind::Petal => s.petal,
            Kind::Pollen => s.pollen,
            _ => 0.0,
        };
        let total: f32 = self.sources.iter().map(weight).sum();
        let mut left = roll * total;
        self.sources.iter().position(|s| {
            left -= weight(s);
            total > 0.0 && left < 0.0
        })
    }
}

/// Returns where a particle of `kind` from `source` starts, in the local frame of the eye: in its crown
/// for a leaf or a petal and low around it for pollen, kept inside the particle box.
pub fn start(source: &Source, kind: Kind, rng: &mut Rng) -> Vec3 {
    let angle = rng.range(0.0, std::f32::consts::TAU);
    let spread = match kind {
        Kind::Pollen => 1.5,
        _ => 1.0,
    };
    let out = source.reach * spread * rng.next().sqrt();
    let rise = match kind {
        Kind::Pollen => rng.range(0.2, 0.6 + source.height),
        _ => source.height * rng.range(0.45, 1.0),
    };
    let at = source.at + Vec3::new(angle.cos() * out, rise, angle.sin() * out);
    let edge = HALF_WIDTH * 0.95;
    Vec3::new(
        at.x.clamp(-edge, edge),
        at.y.clamp(-BELOW, ABOVE),
        at.z.clamp(-edge, edge),
    )
}

#[cfg(test)]
#[path = "tests_shed.rs"]
mod tests;
