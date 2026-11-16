//! Ground classification from height and slope.

use super::SEA_LEVEL;

/// Slope (rise over run) above which ground counts as rock.
const ROCK_SLOPE: f32 = 1.0;

/// Height above sea level above which ground counts as rock.
const ROCK_HEIGHT: f32 = 15.0;

/// Height above sea level below which land counts as sand.
const SAND_HEIGHT: f32 = 1.5;

/// What covers the ground at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// Below sea level.
    Water,
    /// Beach near the shore.
    Sand,
    /// Flat land above the beach.
    Grass,
    /// Steep or high land.
    Rock,
}

/// Returns the surface for a ground height and a slope (rise over run).
pub fn classify(height: f32, slope: f32) -> Surface {
    if height <= SEA_LEVEL {
        Surface::Water
    } else if slope > ROCK_SLOPE || height > SEA_LEVEL + ROCK_HEIGHT {
        Surface::Rock
    } else if height < SEA_LEVEL + SAND_HEIGHT {
        Surface::Sand
    } else {
        Surface::Grass
    }
}
