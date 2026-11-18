//! The record a globe carries: the class of the planet, its sea level, whether it has a sea at all and
//! whether it has ground.

use super::Globe;
use super::datum::datum;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::lands::Lands;
use crate::world::{Planet, SEA_LEVEL};

impl Globe {
    /// Returns this globe as the planet `record` describes: its ground rules, and its sea raised or lowered
    /// to cover the share of the surface the record gives it.
    pub fn with_record(mut self, record: PlanetRecord) -> Self {
        self.record = record;
        self.datum = datum(record.ocean).unwrap_or(0.0);
        self.lands = Lands::of(&record);
        self
    }

    /// Returns the physical record of the planet.
    pub fn record(&self) -> &PlanetRecord {
        &self.record
    }

    /// Returns true when the planet has ground to stand on. A gas giant has none: its deck is a smooth
    /// sphere at the sea level radius, height 0 everywhere, that nothing walks, grows or is built on.
    pub fn has_ground(&self) -> bool {
        self.record.class != PlanetClass::GasGiant
    }

    /// Returns true when a liquid fills the low ground. Without one, ground below height 0 is dry, and a
    /// planet without ground has no sea either.
    pub fn has_sea(&self) -> bool {
        self.has_ground() && self.lands.has_sea()
    }

    /// Returns the height of the liquid surface for comparing ground heights: [`SEA_LEVEL`] on a planet with
    /// a sea, and lower than any ground on a dry one, so that nothing there counts as under the sea.
    pub fn sea_level(&self) -> f32 {
        match self.has_sea() {
            true => SEA_LEVEL,
            false => f32::NEG_INFINITY,
        }
    }

    /// Returns the ground rules of the planet.
    pub(in crate::world) fn lands(&self) -> Lands {
        self.lands
    }
}

impl Planet {
    /// Returns the physical record of the streamed planet.
    pub fn record(&self) -> &PlanetRecord {
        self.globe.record()
    }
}
