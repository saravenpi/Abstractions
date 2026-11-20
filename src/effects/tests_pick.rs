//! Tests of the kind a respawning particle takes from its surroundings.

use super::*;

/// Builds surroundings on land at ground level with `leaves` leafy plants and `flowers` in flower within reach.
fn land(biome: Biome, daylight: f32, wind: f32, (leaves, flowers): (f32, f32)) -> Ambient {
    Ambient {
        biome,
        surface: Surface::Grass,
        height: 5.0,
        daylight,
        wind,
        leaves,
        flowers,
    }
}

/// Returns true when some roll in `[0, 1)` picks `kind` in `ambient`.
fn ever(ambient: &Ambient, kind: Kind) -> bool {
    (0..200).any(|step| pick(ambient, step as f32 / 200.0) == kind)
}

/// Leaves fall where leafy plants stand, petals and pollen where plants flower, and nothing else
/// on bare ground, whatever the biome label says.
#[test]
fn kind_follows_the_plants_nearby() {
    for biome in [Biome::Forest, Biome::Cherry, Biome::Meadow, Biome::Tropical] {
        let bare = land(biome, 1.0, 0.3, (0.0, 0.0));
        for kind in [Kind::Leaf, Kind::Petal, Kind::Pollen] {
            assert!(
                !ever(&bare, kind),
                "{biome:?} without plants drops {kind:?}"
            );
        }
    }
    let wood = land(Biome::Desert, 1.0, 0.3, (4.0, 0.0));
    assert!(ever(&wood, Kind::Leaf) && !ever(&wood, Kind::Petal));
    let bloom = land(Biome::Meadow, 1.0, 0.3, (0.0, 3.0));
    assert!(!ever(&bloom, Kind::Leaf) && ever(&bloom, Kind::Petal) && ever(&bloom, Kind::Pollen));
    let night = land(Biome::Meadow, 0.0, 0.3, (0.0, 3.0));
    assert!(ever(&night, Kind::Petal) && !ever(&night, Kind::Pollen));
}

/// Snow, bubbles and dust stay physical: cold air, water and a desert wind.
#[test]
fn snow_bubbles_and_dust_stay() {
    assert_eq!(
        pick(&land(Biome::Taiga, 1.0, 0.3, (4.0, 0.0)), 0.1),
        Kind::Snow
    );
    let under = Ambient {
        height: -3.0,
        ..land(Biome::Forest, 1.0, 0.3, (4.0, 0.0))
    };
    assert_eq!(pick(&under, 0.1), Kind::Bubble);
    for step in 0..100 {
        let roll = step as f32 / 100.0;
        assert_eq!(
            pick(&land(Biome::Desert, 1.0, 0.2, (0.0, 0.0)), roll),
            Kind::Hidden
        );
    }
    assert_eq!(
        pick(&land(Biome::Desert, 1.0, 0.9, (0.0, 0.0)), 0.1),
        Kind::Dust
    );
}

/// High above every summit the air is empty, whatever lies below.
#[test]
fn nothing_drifts_above_the_ceiling() {
    for biome in [Biome::Cherry, Biome::Taiga, Biome::Forest] {
        let high = Ambient {
            height: CEILING + 1.0,
            ..land(biome, 1.0, 0.9, (5.0, 5.0))
        };
        assert_eq!(pick(&high, 0.1), Kind::Hidden);
    }
}
