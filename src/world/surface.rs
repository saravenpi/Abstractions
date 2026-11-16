//! Ground classification from height, slope and biome, and the spawn search.

use super::SEA_LEVEL;
use super::biome::Biome;

/// Slope (rise over run) above which ground counts as rock.
const ROCK_SLOPE: f32 = 1.0;

/// Height above sea level above which ground counts as rock.
const ROCK_HEIGHT: f32 = 38.0;

/// Height above sea level above which ground counts as snow.
const SNOW_LINE: f32 = 56.0;

/// Height above sea level below which land counts as sand.
const SAND_HEIGHT: f32 = 1.5;

/// Sea floor height below which water is deep.
const SHALLOWS_BOTTOM: f32 = -4.6;

/// Highest sea floor height of a reef.
const REEF_TOP: f32 = -1.0;

/// Lowest sea floor height of a reef.
const REEF_BOTTOM: f32 = -6.5;

/// What covers the ground at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// Deep sea floor.
    Water,
    /// Sandy sea floor in shallow water near a coast.
    Shallows,
    /// Coral reef on the sea floor.
    Reef,
    /// Muddy floor of a small inland pond.
    Pond,
    /// Beach near the shore, and all of the desert.
    Sand,
    /// Flat land above the beach.
    Grass,
    /// Steep or high land.
    Rock,
    /// Snow on cold or very high land.
    Snow,
}

/// Returns the snow line height above sea level for a biome.
fn snow_line(biome: Biome) -> f32 {
    match biome {
        Biome::Taiga => SAND_HEIGHT,
        Biome::Meadow | Biome::Forest | Biome::Cherry => SNOW_LINE,
        Biome::Desert | Biome::Tropical => 90.0,
    }
}

/// Returns the sea floor surface for a height at or below sea level.
fn seabed(height: f32, slope: f32, biome: Biome) -> Surface {
    let reef_slope = match biome {
        Biome::Tropical | Biome::Desert => 0.3,
        Biome::Meadow => 0.5,
        _ => f32::MAX,
    };
    if slope > reef_slope && (REEF_BOTTOM..REEF_TOP).contains(&height) {
        Surface::Reef
    } else if height > SHALLOWS_BOTTOM {
        Surface::Shallows
    } else {
        Surface::Water
    }
}

/// Returns the surface for a ground height, a slope (rise over run) and a biome.
pub fn classify(height: f32, slope: f32, biome: Biome) -> Surface {
    if height <= SEA_LEVEL {
        seabed(height, slope, biome)
    } else if slope > ROCK_SLOPE {
        Surface::Rock
    } else if height > SEA_LEVEL + snow_line(biome) {
        Surface::Snow
    } else if height > SEA_LEVEL + ROCK_HEIGHT {
        Surface::Rock
    } else if biome == Biome::Desert || height < SEA_LEVEL + SAND_HEIGHT {
        Surface::Sand
    } else {
        Surface::Grass
    }
}
