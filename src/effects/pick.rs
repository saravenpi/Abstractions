//! Picks the kind of a respawning particle from the surroundings.

use super::kind::{Ambient, CEILING, Kind};
use super::tune::RESPAWNS;
use crate::world::{Biome, SEA_LEVEL, Surface};

/// Camera height above sea level from which the air counts as snowy.
const SNOW_HEIGHT: f32 = 55.0;

/// Daylight below which the air counts as night.
const NIGHT_DAYLIGHT: f32 = 0.25;

/// Wind strength above which the desert lifts dust and the sea throws spray.
const STRONG_WIND: f32 = 0.5;

/// Returns the first kind whose cumulative share exceeds `roll`, or `Hidden`.
fn choose(roll: f32, shares: &[(Kind, f32)]) -> Kind {
    let mut total = 0.0;
    for &(kind, share) in shares {
        total += share;
        if roll < total {
            return kind;
        }
    }
    Kind::Hidden
}

/// Most of the pool the falling leaves take.
const LEAF_MOST: f32 = 0.45;

/// Most of the pool the falling petals take.
const PETAL_MOST: f32 = 0.5;

/// Most of the pool the pollen takes.
const POLLEN_MOST: f32 = 0.12;

/// Returns the share of the respawns a kind shed at `rate` per second takes, capped at `most`.
fn share(rate: f32, most: f32) -> f32 {
    (rate / RESPAWNS).min(most)
}

/// Picks the kind on land: dust in a desert wind, and only what the plants within reach shed at the rate
/// they shed it: leaves, petals, and pollen by day.
fn on_land(ambient: &Ambient, roll: f32) -> Kind {
    let day = ambient.daylight >= NIGHT_DAYLIGHT;
    let windy = ambient.wind >= STRONG_WIND && ambient.biome == Biome::Desert;
    let pollen = match day {
        true => share(ambient.pollen, POLLEN_MOST),
        false => 0.0,
    };
    choose(
        roll,
        &[
            (Kind::Dust, if windy { 0.4 } else { 0.0 }),
            (Kind::Leaf, share(ambient.leaves, LEAF_MOST)),
            (Kind::Petal, share(ambient.petals, PETAL_MOST)),
            (Kind::Pollen, pollen),
        ],
    )
}

/// Returns true when the camera sits over open sea.
fn over_sea(ambient: &Ambient) -> bool {
    matches!(
        ambient.surface,
        Surface::Water | Surface::Shallows | Surface::Reef
    )
}

/// Returns true when the air is cold enough for snow.
fn snowy(ambient: &Ambient) -> bool {
    ambient.surface == Surface::Snow
        || ambient.biome == Biome::Taiga
        || ambient.height > SNOW_HEIGHT
}

/// Picks the kind of a respawning particle from the surroundings and a roll in `[0, 1)`.
pub fn pick(ambient: &Ambient, roll: f32) -> Kind {
    if ambient.height < SEA_LEVEL {
        choose(roll, &[(Kind::Bubble, 0.4), (Kind::Plankton, 0.6)])
    } else if ambient.height > CEILING {
        Kind::Hidden
    } else if over_sea(ambient) {
        let spray = if ambient.wind >= STRONG_WIND {
            0.15
        } else {
            0.0
        };
        choose(roll, &[(Kind::Spray, spray)])
    } else if snowy(ambient) {
        choose(roll, &[(Kind::Snow, 0.8)])
    } else {
        on_land(ambient, roll)
    }
}

#[cfg(test)]
#[path = "tests_pick.rs"]
mod tests;
