//! Tests of the lands of each class: surfaces, biomes and life.

use super::Lands;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::biome::{Biome, ground_rgb};
use crate::world::surface::{Surface, classify};

use PlanetClass::*;

/// Heights from the deep floor to the summits, and slopes from flat to a cliff.
pub(super) const GROUND: [(f32, f32); 10] = [
    (-30.0, 0.1),
    (-5.5, 0.4),
    (-3.0, 0.6),
    (-0.5, 0.0),
    (0.8, 0.1),
    (6.0, 0.2),
    (20.0, 1.4),
    (45.0, 0.3),
    (70.0, 0.2),
    (110.0, 0.5),
];

/// Returns the lands of `class` with a sea covering the `ocean` share of the planet.
pub(super) fn lands(class: PlanetClass, ocean: f64) -> Lands {
    Lands::of(&PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    })
}

/// A world of seas and islands keeps the surfaces and the palette of home, to the bit.
#[test]
fn seas_and_islands_keep_the_ground_of_home() {
    for lands in [Ocean, Island].map(|class| lands(class, 0.6)) {
        assert!(lands.temperate() && lands.biome().is_none() && lands.ponds());
        for (biome, (height, slope)) in Biome::ALL.into_iter().flat_map(|b| GROUND.map(|g| (b, g)))
        {
            let surface = classify(height, slope, biome);
            assert_eq!(lands.surface(height, slope, biome), surface);
            assert_eq!(
                lands.rgb(surface, biome, height),
                ground_rgb(surface, biome)
            );
        }
        assert_eq!(lands.sea_rgb(), [0.23, 0.6, 0.76]);
        assert_eq!(
            (lands.growth(), lands.sea_life(), lands.settled()),
            (1.0, true, true)
        );
    }
}

/// Without a sea the old sea bed is dry land, and a liquid that is not the water of home has no reef.
#[test]
fn surfaces_follow_the_sea_and_the_class() {
    for class in PlanetClass::ALL {
        let (dry, sea) = (lands(class, 0.0), lands(class, 0.5));
        assert!(!dry.has_sea() && !dry.ponds() && !dry.sea_life());
        for (biome, (height, slope)) in Biome::ALL.into_iter().flat_map(|b| GROUND.map(|g| (b, g)))
        {
            let on_dry = dry.surface(height, slope, biome);
            let under = matches!(on_dry, Surface::Water | Surface::Shallows | Surface::Reef);
            assert!(!under, "{class:?} dry has {on_dry:?} at {height}");
            let on_sea = sea.surface(height, slope, biome);
            let reef = on_sea == Surface::Reef && !sea.temperate();
            assert!(!reef, "{class:?} has a reef at {height}");
            let snow = on_sea == Surface::Snow && matches!(class, Desert | Lava | Barren);
            assert!(!snow, "{class:?} has snow at {height}");
        }
    }
}

/// One biome covers each bare class, snow covers an ice world above its shore, and a desert has no grass.
#[test]
fn bare_classes_have_one_biome() {
    let biomes = [Desert, Ice, Lava, Barren].map(|class| lands(class, 0.0).biome());
    let (sand, cold) = (Some(Biome::Desert), Some(Biome::Taiga));
    assert_eq!(biomes, [sand, cold, sand, sand]);
    let ice = lands(Ice, 0.0);
    assert_eq!(ice.surface(12.0, 0.2, Biome::Taiga), Surface::Snow);
    assert_eq!(ice.surface(-12.0, 0.2, Biome::Taiga), Surface::Sand);
    assert_eq!(ice.surface(12.0, 1.5, Biome::Taiga), Surface::Rock);
    let desert = lands(Desert, 0.0);
    for (height, slope) in GROUND {
        let surface = desert.surface(height, slope, Biome::Desert);
        assert!(
            matches!(surface, Surface::Sand | Surface::Rock),
            "{surface:?}"
        );
    }
}

/// Plants grow thinner on a desert and on ice, not at all on molten or bare rock, where nobody lives either.
#[test]
fn life_follows_the_class() {
    let growth = PlanetClass::ALL.map(|class| lands(class, 0.5).growth());
    assert_eq!(growth, [1.0, 1.0, 0.5, 0.4, 0.0, 0.0, 0.0]);
    let settled = PlanetClass::ALL.map(|class| lands(class, 0.5).settled());
    assert_eq!(settled, [true, true, true, true, false, false, false]);
    let sea_life = PlanetClass::ALL.map(|class| lands(class, 0.5).sea_life());
    assert_eq!(sea_life, [true, true, false, false, false, false, false]);
    let ponds = PlanetClass::ALL.map(|class| lands(class, 0.5).ponds());
    assert_eq!(ponds, [true, true, true, false, false, false, false]);
}
