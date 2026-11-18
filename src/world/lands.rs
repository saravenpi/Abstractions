//! The lands of a planet: what its class and its sea do to the surfaces, the biomes and the life of the ground.

mod life;
mod paint;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_paint;
#[cfg(test)]
mod tests_thrive;
pub(in crate::world) mod thrive;

use super::SEA_LEVEL;
use super::biome::Biome;
use super::surface::{Surface, bare_bed, classify, land};
use crate::universe::{PlanetClass, PlanetRecord};

/// The ground rules of one planet, read from its record.
///
/// The home planet and every world of seas and islands keep the rules the generator was tuned for. The other
/// classes bend them: one biome everywhere, bare sea floors, no snow where it is hot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lands {
    /// The class of the planet.
    class: PlanetClass,
    /// Whether a liquid fills the low ground.
    sea: bool,
}

impl Lands {
    /// Returns the ground rules of a planet with `record`.
    pub fn of(record: &PlanetRecord) -> Self {
        Self {
            class: record.class,
            sea: record.ocean > 0.0,
        }
    }

    /// Returns true when a liquid fills the ground below sea level.
    pub fn has_sea(self) -> bool {
        self.sea
    }

    /// Returns true on a world of seas and islands, whose ground, sea floor and life are the ones of home.
    pub fn temperate(self) -> bool {
        matches!(self.class, PlanetClass::Ocean | PlanetClass::Island)
    }

    /// Returns true when the liquid of the planet is water, as at home.
    pub fn watery(self) -> bool {
        self.temperate() || self.class == PlanetClass::Desert
    }

    /// Returns the one biome that covers the whole planet, or nothing where the climate decides.
    ///
    /// Plants, structures, creatures and sounds all follow the biome, so one answer here fits them all.
    pub fn biome(self) -> Option<Biome> {
        match self.class {
            PlanetClass::Ocean | PlanetClass::Island => None,
            PlanetClass::Ice => Some(Biome::Taiga),
            PlanetClass::Desert
            | PlanetClass::Lava
            | PlanetClass::Barren
            | PlanetClass::GasGiant => Some(Biome::Desert),
        }
    }

    /// Returns true when ponds may lie in the low inland ground: on a planet whose sea is water. The few a
    /// desert keeps are its oases.
    pub fn ponds(self) -> bool {
        self.sea && self.watery()
    }

    /// Returns what covers ground at `height` above the sea level, for a slope (rise over run) and a biome.
    ///
    /// Without a sea the old sea bed is dry land. A liquid that is not the water of home has a bare floor,
    /// and the hot classes have no snow.
    pub fn surface(self, height: f32, slope: f32, biome: Biome) -> Surface {
        if self.sea && height <= SEA_LEVEL {
            return match self.temperate() {
                true => classify(height, slope, biome),
                false => bare_bed(height),
            };
        }
        let hot = !matches!(
            self.class,
            PlanetClass::Ocean | PlanetClass::Island | PlanetClass::Ice
        );
        match land(height, slope, biome) {
            Surface::Snow if hot => Surface::Rock,
            surface => surface,
        }
    }
}
