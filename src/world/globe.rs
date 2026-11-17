//! The globe: one planet as a function of the direction from its centre.

mod chart;
mod cube;
pub mod far;
mod flat;
mod ground;
mod id;
pub mod quadtree;
mod site;
mod sphere;
mod tangent;

#[cfg(test)]
mod tests;

pub use chart::Chart;
pub use cube::Face;
pub use id::ChunkId;
pub(in crate::world) use site::Site;
pub use tangent::Tangent;

use super::biome::Climate;
use super::islands::Shape;
use crate::universe::PlanetSpec;

/// A planet: its size, its pull and the noise fields that shape its ground.
///
/// Shared behind an `Arc` by every [`super::Terrain`] view of the planet.
#[derive(Clone)]
pub struct Globe {
    pub(in crate::world) seed: u64,
    radius: f64,
    gravity: f64,
    pub(in crate::world) shape: Shape,
    pub(in crate::world) climate: Climate,
}

impl Globe {
    /// Builds the globe for a terrain seed, a sea level radius in metres and a surface gravity
    /// in metres per second squared. The same values always give the same planet.
    pub fn new(seed: u64, radius: f64, gravity: f64) -> Self {
        Self {
            seed,
            radius,
            gravity,
            shape: Shape::new(seed),
            climate: Climate::new(seed),
        }
    }

    /// Builds the globe of a planet of the universe.
    pub fn of(spec: &PlanetSpec) -> Self {
        Self::new(spec.seed, spec.radius, spec.gravity)
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
}
