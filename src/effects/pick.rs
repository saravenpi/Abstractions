//! Picks the kind of a respawning particle from the surroundings.

use super::kind::{Ambient, Kind};
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

/// Picks the kind on land from the biome and the time of day.
fn on_land(ambient: &Ambient, roll: f32) -> Kind {
    let night = ambient.daylight < NIGHT_DAYLIGHT;
    let dust = if ambient.wind >= STRONG_WIND {
        0.4
    } else {
        0.0
    };
    match (ambient.biome, night) {
        (Biome::Desert, _) => choose(roll, &[(Kind::Dust, dust)]),
        (Biome::Cherry, false) => choose(roll, &[(Kind::Petal, 0.7), (Kind::Pollen, 0.08)]),
        (Biome::Cherry, true) => choose(roll, &[(Kind::Petal, 0.7)]),
        (Biome::Forest, false) => choose(roll, &[(Kind::Leaf, 0.35)]),
        (Biome::Forest, true) => choose(roll, &[(Kind::Leaf, 0.15)]),
        (Biome::Meadow, false) => choose(roll, &[(Kind::Leaf, 0.2), (Kind::Pollen, 0.1)]),
        (Biome::Meadow, true) => choose(roll, &[(Kind::Leaf, 0.1)]),
        (Biome::Tropical, false) => choose(roll, &[(Kind::BigLeaf, 0.12)]),
        (Biome::Tropical, true) => choose(roll, &[(Kind::BigLeaf, 0.05)]),
        (Biome::Taiga, _) => Kind::Hidden,
    }
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
mod tests {
    use super::*;

    /// Builds surroundings on land at ground level.
    fn land(biome: Biome, daylight: f32, wind: f32) -> Ambient {
        Ambient {
            biome,
            surface: Surface::Grass,
            height: 5.0,
            daylight,
            wind,
        }
    }

    /// Every biome yields its signature particle and calm deserts yield nothing.
    #[test]
    fn kind_follows_the_surroundings() {
        assert_eq!(pick(&land(Biome::Cherry, 1.0, 0.3), 0.1), Kind::Petal);
        assert_eq!(pick(&land(Biome::Taiga, 1.0, 0.3), 0.1), Kind::Snow);
        assert_eq!(pick(&land(Biome::Forest, 1.0, 0.3), 0.1), Kind::Leaf);
        let under = Ambient {
            height: -3.0,
            ..land(Biome::Forest, 1.0, 0.3)
        };
        assert_eq!(pick(&under, 0.1), Kind::Bubble);
        for step in 0..100 {
            let roll = step as f32 / 100.0;
            assert_eq!(pick(&land(Biome::Desert, 1.0, 0.2), roll), Kind::Hidden);
        }
        assert_eq!(pick(&land(Biome::Forest, 0.0, 0.3), 0.1), Kind::Leaf);
        assert_eq!(pick(&land(Biome::Desert, 1.0, 0.9), 0.1), Kind::Dust);
    }
}
