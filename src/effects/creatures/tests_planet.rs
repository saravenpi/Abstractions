//! Tests of which creatures each class of planet and each rung of its life lets out.

use super::habitat::{Species, presence, presence_on};
use crate::effects::kind::Ambient;
use crate::life::{Chemistry, Energy, Life, Rung, Solvent, Structure};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::{Biome, Globe, Surface};

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

/// Returns a globe of `record` carrying `life`: none given, a dead planet, or life at a rung.
fn globe(record: PlanetRecord, life: Option<Option<Rung>>) -> Globe {
    let globe = Globe::new(1, 3000.0, 22.0).with_record(record);
    let Some(rung) = life else {
        return globe;
    };
    let chemistry = Chemistry {
        solvent: Solvent::Water,
        energy: Energy::Light,
        structure: Structure::Carbon,
        pigment: [0.2, 0.6, 0.2],
        glow: [0.2, 0.8, 0.6],
    };
    globe.with_life(rung.map(|rung| Life {
        rung,
        chemistry,
        species: Vec::new(),
        relations: Vec::new(),
        keystone: None,
    }))
}

/// Owner decision, 2026-11-19: no planet is special. A temperate globe whose life reached the skeletal
/// rung lets every species out exactly as the surroundings call for, the same globe given no life keeps
/// those rules too, and the same globe dead lets nothing out. Insects need shelled life and birds
/// skeletal life: mats and soft life let nothing out, shelled life lets insects out but no bird.
#[test]
fn creatures_follow_life() {
    let living = |rung| globe(PlanetRecord::TEMPERATE, Some(Some(rung)));
    let (skeletal, unknown) = (living(Rung::Skeletal), globe(PlanetRecord::TEMPERATE, None));
    let (dead, mats, soft) = (
        globe(PlanetRecord::TEMPERATE, Some(None)),
        living(Rung::Mats),
        living(Rung::Soft),
    );
    for species in SPECIES {
        for a in [meadow(0.0), meadow(1.0), beach()] {
            assert_eq!(presence_on(&skeletal, species, &a), presence(species, &a));
            assert_eq!(presence_on(&unknown, species, &a), presence(species, &a));
            for barren in [&dead, &mats, &soft] {
                assert_eq!(presence_on(barren, species, &a), 0.0, "{species:?}");
            }
        }
    }
    assert!(presence_on(&skeletal, Species::Butterfly, &meadow(1.0)) > 0.0);
    let shelled = living(Rung::Shelled);
    assert!(presence_on(&shelled, Species::Butterfly, &meadow(1.0)) > 0.0);
    assert!(presence_on(&shelled, Species::Firefly, &meadow(0.0)) > 0.0);
    assert_eq!(presence_on(&shelled, Species::Crow, &meadow(1.0)), 0.0);
    assert_eq!(presence_on(&shelled, Species::Gull, &beach()), 0.0);
}

/// Nothing flies where the air pressure is under a hundredth of home, on any class.
#[test]
fn nothing_flies_without_air() {
    for class in PlanetClass::ALL {
        let thin = globe(record(class, 0.009, 0.6), None);
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
    for class in [Lava, Barren] {
        let bare = globe(record(class, 1.0, 0.3), None);
        for species in SPECIES {
            for a in [meadow(0.0), meadow(1.0), beach()] {
                assert_eq!(presence_on(&bare, species, &a), 0.0);
            }
        }
    }
    for class in [Ocean, Island, Desert, Ice] {
        let living = globe(record(class, 0.4, 0.6), None);
        assert!(presence_on(&living, Species::Crow, &meadow(1.0)) > 0.0);
    }
}

/// Gulls need a sea, and a creature in the dry basin of a world without one is not under water.
#[test]
fn a_dry_world_has_no_gulls_and_no_drowned_basin() {
    let (home, dry) = (
        globe(PlanetRecord::TEMPERATE, None),
        globe(record(PlanetClass::Desert, 0.4, 0.0), None),
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
