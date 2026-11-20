//! Class tests: one hand-built record per branch of the class rule, and the look table order.

use super::class::{MOLTEN, VOLCANIC, classify, liquid};
use super::climate::Climate;
use super::*;

/// No air to speak of is barren, whatever the heat and the water.
#[test]
fn no_air_is_barren() {
    for (temperature, water) in [(288.0, Water::Liquid), (2000.0, Water::None)] {
        let record = PlanetRecord {
            pressure: 0.009,
            temperature,
            water,
            ..PlanetRecord::TEMPERATE
        };
        assert_eq!(classify(&record), PlanetClass::Barren);
    }
}

/// A body whose tides give off more than a watt per square metre is a lava world, air or not,
/// with lava over 10 to 40 percent of its ground. A weaker tide changes nothing.
#[test]
fn volcanic_moons_are_lava() {
    let record = |tide, pressure| PlanetRecord {
        tide,
        pressure,
        temperature: 110.0,
        water: Water::Ice,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(classify(&record(VOLCANIC, 0.0)), PlanetClass::Lava);
    assert_eq!(classify(&record(VOLCANIC, 1.0)), PlanetClass::Lava);
    assert_eq!(classify(&record(0.99 * VOLCANIC, 0.0)), PlanetClass::Barren);
    assert_eq!(classify(&record(0.99 * VOLCANIC, 1.0)), PlanetClass::Ice);
    let cold = Climate {
        albedo: 0.5,
        greenhouse: 0.0,
        temperature: 110.0,
        water: Water::Ice,
    };
    assert_eq!(liquid(0.0, &cold, 0.5, 0.0), 0.0);
    assert_eq!(liquid(0.0, &cold, 0.5, VOLCANIC), 0.1);
    assert!((liquid(0.0, &cold, 0.5, 2.5 * VOLCANIC) - 0.25).abs() < 1e-12);
    assert!((liquid(0.0, &cold, 0.5, 40.0 * VOLCANIC) - 0.4).abs() < 1e-12);
}

/// Air over rock hot enough to melt is lava, from the melting point up.
#[test]
fn molten_rock_is_lava() {
    let record = |temperature| PlanetRecord {
        temperature,
        water: Water::Steam,
        ocean: 0.1,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(classify(&record(MOLTEN)), PlanetClass::Lava);
    assert_eq!(classify(&record(MOLTEN - 1.0)), PlanetClass::Desert);
}

/// Liquid water over 85 percent of the surface and more is an ocean world.
#[test]
fn drowned_land_is_ocean() {
    let record = PlanetRecord {
        ocean: 0.85,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(classify(&record), PlanetClass::Ocean);
}

/// Liquid water with land left is an island world, and the home record is one.
#[test]
fn seas_and_land_are_island() {
    let record = PlanetRecord {
        ocean: 0.84,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(classify(&record), PlanetClass::Island);
    assert_eq!(classify(&PlanetRecord::TEMPERATE), PlanetClass::Island);
}

/// Frozen water under air is an ice world.
#[test]
fn frozen_water_is_ice() {
    let record = PlanetRecord {
        temperature: 200.0,
        water: Water::Ice,
        ocean: 0.0,
        ..PlanetRecord::TEMPERATE
    };
    assert_eq!(classify(&record), PlanetClass::Ice);
}

/// Air without open water is desert, be the water missing or boiled away.
#[test]
fn air_without_water_is_desert() {
    for water in [Water::None, Water::Steam] {
        let record = PlanetRecord {
            water,
            ocean: 0.0,
            ..PlanetRecord::TEMPERATE
        };
        assert_eq!(classify(&record), PlanetClass::Desert);
    }
}

/// The look table follows the class order: only lava glows, and only barren worlds and gas
/// giants, whose banded deck is their cloud, have no separate cloud layer.
#[test]
fn looks_follow_the_class_order() {
    for class in PlanetClass::ALL {
        let look = class.look();
        assert_eq!(look.glow > 0.0, class == PlanetClass::Lava, "{class:?}");
        assert_eq!(
            look.clouds == 0.0,
            matches!(class, PlanetClass::Barren | PlanetClass::GasGiant),
            "{class:?}"
        );
    }
    let labels = PlanetClass::ALL.map(PlanetClass::label);
    assert_eq!(
        labels,
        [
            "ocean",
            "island",
            "desert",
            "ice",
            "lava",
            "barren",
            "gas giant"
        ]
    );
    assert_eq!(PlanetClass::Ice.look().land.x, 0.80);
}
