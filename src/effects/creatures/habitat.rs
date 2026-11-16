//! Which creatures are out for a biome, a surface and the time of day.

use super::anim::ramp;
use crate::effects::kind::Ambient;
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

/// Returns the share of a species' pool that should be out, from 0 to 1.
pub fn presence(species: Species, a: &Ambient) -> f32 {
    if a.height < SEA_LEVEL {
        return 0.0;
    }
    let day = ramp(0.35, 0.75, a.daylight);
    let night = 1.0 - ramp(0.1, 0.4, a.daylight);
    match species {
        Species::Firefly => night * firefly_haunt(a),
        Species::Butterfly => day * butterfly_haunt(a),
        Species::Crow => day * crow_haunt(a),
        Species::Gull => day * gull_haunt(a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds surroundings above the sea.
    fn spot(biome: Biome, surface: Surface, daylight: f32) -> Ambient {
        Ambient {
            biome,
            surface,
            height: 4.0,
            daylight,
            wind: 0.3,
        }
    }

    /// Each species keeps to its hours and haunts.
    #[test]
    fn creatures_keep_their_haunts() {
        let night = |b, s| presence(Species::Firefly, &spot(b, s, 0.0));
        assert_eq!(night(Biome::Forest, Surface::Grass), 1.0);
        assert_eq!(night(Biome::Desert, Surface::Pond), 1.0);
        assert_eq!(night(Biome::Desert, Surface::Sand), 0.0);
        assert_eq!(night(Biome::Forest, Surface::Water), 0.0);
        assert_eq!(
            presence(Species::Firefly, &spot(Biome::Forest, Surface::Grass, 1.0)),
            0.0
        );
        let day = |sp, b, s| presence(sp, &spot(b, s, 1.0));
        assert_eq!(day(Species::Butterfly, Biome::Meadow, Surface::Grass), 1.0);
        assert_eq!(day(Species::Butterfly, Biome::Desert, Surface::Sand), 0.0);
        assert_eq!(day(Species::Crow, Biome::Meadow, Surface::Water), 0.0);
        assert_eq!(day(Species::Gull, Biome::Meadow, Surface::Water), 1.0);
        assert!(day(Species::Crow, Biome::Taiga, Surface::Grass) < 0.5);
        let under = Ambient {
            height: -2.0,
            ..spot(Biome::Meadow, Surface::Grass, 1.0)
        };
        assert_eq!(presence(Species::Crow, &under), 0.0);
    }
}
