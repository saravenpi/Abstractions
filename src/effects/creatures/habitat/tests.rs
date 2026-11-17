//! Tests of where each creature lives.

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

/// No creature is out for a camera above the ceiling, by day or by night.
#[test]
fn creatures_stay_under_the_ceiling() {
    for daylight in [0.0, 1.0] {
        let high = Ambient {
            height: CEILING,
            ..spot(Biome::Meadow, Surface::Grass, daylight)
        };
        let all = [
            Species::Firefly,
            Species::Butterfly,
            Species::Crow,
            Species::Gull,
        ];
        assert!(
            all.into_iter()
                .all(|species| presence(species, &high) == 0.0)
        );
    }
}
