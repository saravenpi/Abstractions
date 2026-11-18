//! The globe: one planet as a function of the direction from its centre.

mod chart;
mod cube;
mod datum;
pub mod far;
mod flat;
mod ground;
mod id;
pub mod quadtree;
mod record;
mod reserve;
mod site;
mod sphere;
mod tangent;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_datum;
#[cfg(test)]
pub(in crate::world) mod tests_deck;
#[cfg(test)]
mod tests_record;

pub use chart::Chart;
pub use cube::Face;
pub use id::ChunkId;
pub use reserve::Reserve;
pub(in crate::world) use site::Site;
pub use tangent::Tangent;

use super::biome::Climate;
use super::islands::Shape;
use super::lands::Lands;
use crate::universe::{PlanetRecord, PlanetSpec};

/// A planet: its size, its pull, its record and the noise fields that shape its ground.
///
/// Shared behind an `Arc` by every [`super::Terrain`] view of the planet.
#[derive(Clone)]
pub struct Globe {
    pub(in crate::world) seed: u64,
    radius: f64,
    gravity: f64,
    pub(in crate::world) shape: Shape,
    pub(in crate::world) climate: Climate,
    reserve: Option<Reserve>,
    /// What the physics says about the planet, and its class.
    record: PlanetRecord,
    /// Raw ground height of the sea level, in metres: what every height is measured from.
    datum: f32,
    /// The ground rules the record gives.
    lands: Lands,
    /// Metres from the centre past which the sky draws the planet instead of the world.
    handover: f64,
}

impl Globe {
    /// Builds the globe for a terrain seed, a sea level radius in metres and a surface gravity
    /// in metres per second squared, with the record of the home planet. The same values always give
    /// the same planet.
    pub fn new(seed: u64, radius: f64, gravity: f64) -> Self {
        let record = PlanetRecord::TEMPERATE;
        Self {
            seed,
            radius,
            gravity,
            shape: Shape::new(seed),
            climate: Climate::new(seed),
            reserve: None,
            record,
            datum: 0.0,
            lands: Lands::of(&record),
            handover: far::handover(radius, None),
        }
    }

    /// Builds the globe of a planet of the universe, with its own record and the hand-over
    /// its size and its rings give.
    pub fn of(spec: &PlanetSpec) -> Self {
        Self {
            handover: far::handover(spec.radius, spec.rings),
            ..Self::new(spec.seed, spec.radius, spec.gravity).with_record(spec.record)
        }
    }

    /// Returns the seed the ground is generated from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the sea level radius in metres.
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Returns the surface gravity in metres per second squared.
    pub fn gravity(&self) -> f64 {
        self.gravity
    }

    /// Returns the metres from the centre past which the sky draws the planet, see
    /// [`far::handover`].
    pub fn handover(&self) -> f64 {
        self.handover
    }
}
