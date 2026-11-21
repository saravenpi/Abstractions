//! Biome rules: which of the six biomes a climate gives, by an ordered list over temperature and
//! wetness, and where snow lies.
//!
//! Rules after the Whittaker style table of davudk, procedural-world-generation,
//! <https://github.com/davudk/procedural-world-generation/blob/main/docs/research/03-climate-biomes.md>.

use super::Climate;
use crate::world::Biome;

/// Kelvin under which the land is taiga.
///
/// Thresholds are tuned on the home planet (288 K, ocean 0.6) with `inland = base / 0.45`: taiga past about
/// 60 degrees, tropical and desert below about 19, forest and cherry from 30 to 60, meadow in the
/// dry belt between.
const COLD: f32 = 267.0;

/// Kelvin over which the land is desert or tropical.
pub(super) const HOT: f32 = 299.0;

/// Wetness under which hot land is desert.
const ARID: f32 = 0.4;

/// Wetness over which temperate land is forest.
const WET: f32 = 0.3;

/// Wetness under which cool temperate land is a cherry grove.
const DRY: f32 = 0.25;

/// Kelvin under which dry temperate land is a cherry grove.
const COOL: f32 = 286.0;

/// Kelvin the jitter moves the temperature by at most.
const JITTER_HEAT: f32 = 1.5;

/// Scale of the jitter on the wetness: it is multiplied by `exp(0.2 * jitter)`.
const JITTER_WET: f32 = 0.2;

/// Kelvin under which the ground is under snow.
pub const SNOW_LINE: f32 = 271.0;

/// Returns the biome of `climate`, `jitter` from -1 to 1 blurring the borders.
///
/// The first rule that holds wins: cold land is taiga, hot dry land desert, hot wet land
/// tropical, wet temperate land forest, dry cool land a cherry grove, and the rest meadow.
/// The jitter never picks a biome: it moves the temperature by up to 1.5 K and scales the
/// wetness by `exp(0.2 * jitter)`.
pub fn biome(climate: &Climate, jitter: f32) -> Biome {
    let heat = climate.temperature + JITTER_HEAT * jitter;
    let wet = climate.moisture * (JITTER_WET * jitter).exp();
    if heat < COLD {
        Biome::Taiga
    } else if heat > HOT && wet < ARID {
        Biome::Desert
    } else if heat > HOT {
        Biome::Tropical
    } else if wet > WET {
        Biome::Forest
    } else if wet < DRY && heat < COOL {
        Biome::Cherry
    } else {
        Biome::Meadow
    }
}

/// Returns true when snow lies at `climate`: under [`SNOW_LINE`].
pub fn snow(climate: &Climate) -> bool {
    climate.temperature < SNOW_LINE
}
