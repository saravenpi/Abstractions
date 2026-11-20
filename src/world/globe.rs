//! The globe: one planet as a function of the direction from its centre.

mod atlas;
mod chart;
mod climate;
mod craters;
mod cube;
mod datum;
mod elevation;
pub mod far;
mod flat;
mod flows;
mod grain;
mod ground;
mod id;
mod landform;
mod life;
mod plates;
pub mod quadtree;
mod record;
mod relief;
mod reserve;
mod roll;
mod scale;
mod season;
mod site;
mod sphere;
#[cfg(test)]
mod tally;
mod tangent;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_climate;
#[cfg(test)]
mod tests_coast;
#[cfg(test)]
mod tests_datum;
#[cfg(test)]
pub(in crate::world) mod tests_deck;
#[cfg(test)]
mod tests_flows;
#[cfg(test)]
mod tests_landform;
#[cfg(test)]
mod tests_planet;
#[cfg(test)]
mod tests_planet_cost;
#[cfg(test)]
mod tests_reach;
#[cfg(test)]
mod tests_record;
#[cfg(test)]
mod tests_relief;
#[cfg(test)]
mod tests_scales;

pub use chart::{BLOCK, Chart};
pub use cube::Face;
pub use grain::Grain;
pub use id::ChunkId;
pub use plates::{Boundary, Crust, Interior, Plates, Sample as PlateSample};
pub use reserve::Reserve;
pub use season::MEAN_SEASON;
pub(in crate::world) use site::Site;
pub use tangent::Tangent;

use std::sync::Arc;

use atlas::Atlas;
use relief::Relief;
use season::Season;

use super::biome::Climate;
use super::islands::Shape;
use super::lands::Lands;
use crate::climate::Setting;
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
    /// Where the sea stands in the local ground, and the large shape of the ground on a planet big enough
    /// for plates.
    relief: Relief,
    /// The ground rules the record gives.
    lands: Lands,
    /// The planet as its climate sees it.
    setting: Setting,
    /// How far round its orbit the planet is now, for the climate of the moment.
    season: Season,
    /// The coarse cells of the sphere, their height grids and their flows, shared by every copy.
    atlas: Arc<Atlas>,
    /// The life of the planet, shared by every copy: none until given, then none on a dead world.
    life: Option<Option<Arc<crate::life::Life>>>,
}

impl Globe {
    /// Builds the globe for a terrain seed, a sea level radius in metres and a surface gravity
    /// in metres per second squared, with the record of the home planet. The same values always give
    /// the same planet.
    pub fn new(seed: u64, radius: f64, gravity: f64) -> Self {
        let record = PlanetRecord::TEMPERATE;
        let (gain, continent) = scale::scale(radius, gravity);
        Self {
            seed,
            radius,
            gravity,
            shape: Shape::new(seed).scaled(gain, continent),
            climate: Climate::new(seed),
            reserve: None,
            record,
            relief: Relief::of(seed, [radius, gravity], &record, None),
            lands: Lands::of(&record),
            setting: climate::setting(seed, radius, gravity),
            season: Season::default(),
            atlas: Arc::new(Atlas::new(0, 1)),
            life: None,
        }
        .charted()
    }

    /// Builds the globe of a planet of the universe, with its own record.
    pub fn of(spec: &PlanetSpec) -> Self {
        Self {
            setting: Setting::of(spec),
            ..Self::new(spec.seed, spec.radius, spec.gravity).with_record(spec.record)
        }
        .charted()
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
        far::handover(self.radius)
    }

    /// Returns the planet as its climate sees it: the setting of its spec, or the home record, day and
    /// orbit for a globe built without one.
    pub fn setting(&self) -> &Setting {
        &self.setting
    }
}
