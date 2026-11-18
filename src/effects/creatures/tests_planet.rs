//! Tests of which creatures each class of planet lets out.

use super::habitat::{Species, presence, presence_on};
use crate::effects::kind::Ambient;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::{Biome, Surface};

/// Every species of creature.
const SPECIES: [Species; 4] = [
    Species::Firefly,
    Species::Butterfly,
    Species::Crow,
    Species::Gull,
];

/// Returns surroundings on a meadow by day or by night.
fn meadow(daylight: f32) -> Ambient {
    Ambient {
        biome: Biome::Meadow,
        surface: Surface::Grass,
        height: 5.0,
        daylight,
        wind: 0.2,
    }
}

/// Returns surroundings on a beach by day.
fn beach() -> Ambient {
    Ambient {
        surface: Surface::Sand,
        ..meadow(1.0)
    }
}

/// Returns the home record changed into `class` with `pressure` and an `ocean` share.
fn record(class: PlanetClass, pressure: f64, ocean: f64) -> PlanetRecord {
    PlanetRecord {
        class,
        pressure,
        ocean,
        ..PlanetRecord::TEMPERATE
    }
}

/// At home every species is out exactly as the surroundings alone call for.
#[test]
fn home_creatures_are_unchanged() {
    let home = PlanetRecord::TEMPERATE;
    for species in SPECIES {
        for a in [meadow(0.0), meadow(1.0), beach()] {
            assert_eq!(presence_on(&home, species, &a), presence(species, &a));
        }
    }
    assert!(presence_on(&home, Species::Butterfly, &meadow(1.0)) > 0.0);
}

/// Nothing flies where the air pressure is under a hundredth of home, on any class.
#[test]
fn nothing_flies_without_air() {
    for class in PlanetClass::ALL {
        let thin = record(class, 0.009, 0.6);
        for species in SPECIES {
            for a in [meadow(0.0), meadow(1.0), beach()] {
                let out = presence_on(&thin, species, &a);
                assert_eq!(out, 0.0, "{species:?} on {class:?}");
            }
        }
    }
}

/// Molten and bare rock have no creature even with air, and the living classes keep theirs.
#[test]
fn creatures_follow_the_class() {
    use PlanetClass::*;
    for species in SPECIES {
        for a in [meadow(0.0), meadow(1.0), beach()] {
            for class in [Lava, Barren] {
                assert_eq!(presence_on(&record(class, 1.0, 0.3), species, &a), 0.0);
            }
        }
    }
    for class in [Ocean, Island, Desert, Ice] {
        let living = record(class, 0.4, 0.6);
        assert!(presence_on(&living, Species::Crow, &meadow(1.0)) > 0.0);
    }
}

/// Gulls need a sea, and a creature in the dry basin of a world without one is not under water.
#[test]
fn a_dry_world_has_no_gulls_and_no_drowned_basin() {
    let (home, dry) = (
        PlanetRecord::TEMPERATE,
        record(PlanetClass::Desert, 0.4, 0.0),
    );
    assert!(presence_on(&home, Species::Gull, &beach()) > 0.0);
    assert_eq!(presence_on(&dry, Species::Gull, &beach()), 0.0);
    let basin = Ambient {
        height: -12.0,
        ..meadow(1.0)
    };
    assert_eq!(presence_on(&home, Species::Crow, &basin), 0.0);
    assert!(presence_on(&dry, Species::Crow, &basin) > 0.0);
}
