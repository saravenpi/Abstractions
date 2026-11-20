//! Tests of what each class of planet lets drift and fly.

use super::*;
use crate::world::{Biome, Surface};

/// Returns surroundings in a snowy place `height` metres above sea level, in a strong wind.
fn place(height: f32) -> Ambient {
    Ambient {
        biome: Biome::Taiga,
        surface: Surface::Snow,
        height,
        daylight: 1.0,
        wind: 0.9,
        leaves: 0.0,
        petals: 0.0,
        pollen: 0.0,
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

/// The home planet drifts exactly what the surroundings alone call for, whatever the roll.
#[test]
fn home_air_is_unchanged() {
    for step in 0..50 {
        let roll = step as f32 / 50.0;
        for height in [-3.0, 5.0, 80.0] {
            let got = drift(&PlanetRecord::TEMPERATE, &place(height), roll);
            assert_eq!(got, pick(&place(height), roll));
        }
    }
}

/// Nothing drifts without air, on any class.
#[test]
fn no_air_carries_nothing() {
    for class in PlanetClass::ALL {
        let thin = record(class, 0.005, 0.6);
        assert!(airless(&thin));
        assert_eq!(drift(&thin, &place(5.0), 0.1), Kind::Hidden);
        assert_eq!(drift(&thin, &place(-3.0), 0.1), Kind::Hidden);
    }
}

/// Snow falls on the cold classes only, and bubbles rise only in water.
#[test]
fn snow_and_bubbles_follow_the_class() {
    use PlanetClass::*;
    for class in PlanetClass::ALL {
        let wet = record(class, 1.0, 0.6);
        let snows = drift(&wet, &place(5.0), 0.1) == Kind::Snow;
        assert_eq!(snows, matches!(class, Ocean | Island | Ice), "{class:?}");
        let bubbles = drift(&wet, &place(-3.0), 0.1) == Kind::Bubble;
        assert_eq!(
            bubbles,
            matches!(class, Ocean | Island | Desert),
            "{class:?}"
        );
    }
}

/// Low dry ground is not under a sea: no bubble rises in the basin of a dry world.
#[test]
fn a_dry_basin_has_no_bubbles() {
    for class in PlanetClass::ALL {
        let dry = record(class, 1.0, 0.0);
        assert_eq!(seen(&dry, &place(-12.0)).height, SEA_LEVEL);
        for step in 0..20 {
            let kind = drift(&dry, &place(-12.0), step as f32 / 20.0);
            assert!(!kind.is_submerged(), "{class:?} drifts {kind:?}");
        }
    }
    assert_eq!(seen(&PlanetRecord::TEMPERATE, &place(-12.0)).height, -12.0);
}

/// A gas giant has no ground to shed leaves, petals, pollen or dust and no water: nothing drifts there
/// in any surroundings, though it has air and no creature lives there.
#[test]
fn a_giant_carries_nothing() {
    let giant = record(PlanetClass::GasGiant, 1.0, 0.6);
    assert!(!airless(&giant));
    let meadow = Ambient {
        biome: Biome::Meadow,
        surface: Surface::Grass,
        ..place(5.0)
    };
    for step in 0..40 {
        let roll = step as f32 / 40.0;
        for ambient in [place(5.0), place(-3.0), meadow] {
            assert_eq!(drift(&giant, &ambient, roll), Kind::Hidden);
        }
    }
}
