//! Which creatures are out for a biome, a surface and the time of day.

use super::anim::ramp;
use crate::effects::kind::{Ambient, CEILING};
use crate::world::{Biome, SEA_LEVEL, Surface};

/// The kinds of wandering creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Species {
    /// Glowing night insect.
    Firefly,
    /// Day insect of meadows and groves.
    Butterfly,
    /// Dark land bird.
    Crow,
    /// White sea bird.
    Gull,
}

/// Returns true over open sea.
fn over_sea(surface: Surface) -> bool {
    matches!(surface, Surface::Water | Surface::Shallows | Surface::Reef)
}

/// Share of fireflies that haunt the place, ignoring the hour.
fn firefly_haunt(a: &Ambient) -> f32 {
    let woods = matches!(
        a.biome,
        Biome::Forest | Biome::Meadow | Biome::Tropical | Biome::Cherry
    );
    match (over_sea(a.surface), a.surface == Surface::Pond, woods) {
        (true, _, _) => 0.0,
        (_, true, _) | (_, _, true) => 1.0,
        _ => 0.0,
    }
}

/// Share of butterflies that haunt the place, ignoring the hour.
fn butterfly_haunt(a: &Ambient) -> f32 {
    if over_sea(a.surface) || a.surface == Surface::Snow {
        return 0.0;
    }
    match a.biome {
        Biome::Meadow | Biome::Cherry | Biome::Tropical => 1.0,
        Biome::Forest => 0.6,
        Biome::Desert | Biome::Taiga => 0.0,
    }
}

/// Share of crows that haunt the place, ignoring the hour.
fn crow_haunt(a: &Ambient) -> f32 {
    match (over_sea(a.surface), a.biome) {
        (true, _) => 0.0,
        (_, Biome::Desert | Biome::Taiga) => 0.35,
        _ => 1.0,
    }
}

/// Share of gulls that haunt the place, ignoring the hour.
fn gull_haunt(a: &Ambient) -> f32 {
    match (over_sea(a.surface), a.surface, a.biome) {
        (true, _, _) => 1.0,
        (_, Surface::Sand, Biome::Desert) => 0.0,
        (_, Surface::Sand, _) => 0.8,
        _ => 0.0,
    }
}

/// Returns the share of a species' pool that should be out, from 0 to 1: none under the sea
/// and none above the ceiling of the air.
pub fn presence(species: Species, a: &Ambient) -> f32 {
    if a.height < SEA_LEVEL {
        return 0.0;
    }
    let low = 1.0 - ramp(CEILING - 100.0, CEILING, a.height);
    let day = ramp(0.35, 0.75, a.daylight) * low;
    let night = (1.0 - ramp(0.1, 0.4, a.daylight)) * low;
    match species {
        Species::Firefly => night * firefly_haunt(a),
        Species::Butterfly => day * butterfly_haunt(a),
        Species::Crow => day * crow_haunt(a),
        Species::Gull => day * gull_haunt(a),
    }
}

#[cfg(test)]
mod tests;
