//! Computer tests: the planet record in plain words, one helper at a time.

use std::f64::consts::PI;

use super::words::{AIR, LONG_YEAR, air, flux, state, temperature, tilt, water, year, zone};
use crate::universe::{PlanetClass, PlanetRecord, Water, Zone};

/// Temperatures are whole degrees Celsius, rounded to the nearest, with a sign below freezing
/// and never a negative zero.
#[test]
fn temperatures_read_in_celsius() {
    let cases = [
        (288.0, "15 C"),
        (287.15, "14 C"),
        (273.15, "0 C"),
        (273.0, "0 C"),
        (273.65, "1 C"),
        (272.65, "-1 C"),
        (153.15, "-120 C"),
        (0.0, "-273 C"),
        (1473.15, "1200 C"),
    ];
    for (kelvin, shown) in cases {
        assert_eq!(temperature(kelvin), shown, "{kelvin}");
    }
    assert_eq!(temperature(PlanetRecord::TEMPERATE.temperature), "15 C");
}

/// The air is none under a hundredth of home, thin under half, like home under one and a
/// half, and thick from there, and the pressure number follows the word.
#[test]
fn air_words_follow_the_thresholds() {
    let cases = [
        (0.0, "none, pressure 0.00"),
        (0.004, "none, pressure 0.00"),
        (0.01, "thin, pressure 0.01"),
        (0.4, "thin, pressure 0.40"),
        (0.5, "like home, pressure 0.50"),
        (1.0, "like home, pressure 1.00"),
        (1.49, "like home, pressure 1.49"),
        (1.5, "thick, pressure 1.50"),
        (92.0, "thick, pressure 92.0"),
        (250.0, "thick, pressure 250"),
    ];
    for (pressure, shown) in cases {
        assert_eq!(air(pressure), shown, "{pressure}");
    }
    assert_eq!(AIR.map(|(limit, _)| limit), [0.01, 0.5, 1.5]);
    assert_eq!(air(-1.0), "none, pressure 0.00");
}

/// Builds a record of `class` whose water is in `state` and whose liquid covers `ocean`.
fn wet(class: PlanetClass, state: Water, ocean: f64) -> PlanetRecord {
    PlanetRecord {
        class,
        water: state,
        ocean,
        ..PlanetRecord::TEMPERATE
    }
}

/// Water reads as its state on every class but lava, and liquid water adds the share of the
/// surface under the oceans in whole percent.
#[test]
fn water_reads_per_state_and_class() {
    let lava = PlanetClass::Lava;
    for class in PlanetClass::ALL.into_iter().filter(|class| *class != lava) {
        assert_eq!(water(&wet(class, Water::None, 0.3)), "none", "{class:?}");
        assert_eq!(water(&wet(class, Water::Ice, 0.3)), "ice", "{class:?}");
        assert_eq!(water(&wet(class, Water::Steam, 0.3)), "steam", "{class:?}");
        let liquid = water(&wet(class, Water::Liquid, 0.6));
        assert_eq!(liquid, "liquid, oceans 60 %", "{class:?}");
    }
    let shares = [
        (0.0, "0 %"),
        (0.956, "96 %"),
        (1.0, "100 %"),
        (1.7, "100 %"),
    ];
    for (ocean, share) in shares {
        let shown = water(&wet(PlanetClass::Ocean, Water::Liquid, ocean));
        assert_eq!(shown, format!("liquid, oceans {share}"));
    }
    let words = [Water::None, Water::Ice, Water::Liquid, Water::Steam].map(state);
    assert_eq!(words, ["none", "ice", "liquid", "steam"]);
    assert_eq!(water(&PlanetRecord::TEMPERATE), "liquid, oceans 60 %");
}

/// A lava world says its molten share whatever state its water is in.
#[test]
fn a_lava_world_reads_its_molten_share() {
    let lava = PlanetClass::Lava;
    assert_eq!(
        water(&wet(lava, Water::None, 0.25)),
        "none, molten rock 25 %"
    );
    assert_eq!(water(&wet(lava, Water::Ice, 0.0)), "ice, molten rock 0 %");
    assert_eq!(
        water(&wet(lava, Water::Steam, 0.404)),
        "steam, molten rock 40 %"
    );
    assert_eq!(
        water(&wet(lava, Water::Liquid, 2.0)),
        "liquid, molten rock 100 %"
    );
}

/// Light reads against home with three digits, and the zone says whether water can be liquid.
#[test]
fn light_and_zone_read_against_home() {
    let cases = [
        (1.0, "1.00 x home"),
        (0.004, "0.00 x home"),
        (0.25, "0.25 x home"),
        (12.5, "12.5 x home"),
        (1500.0, "1500 x home"),
        (-1.0, "0.00 x home"),
    ];
    for (light, shown) in cases {
        assert_eq!(flux(light), shown, "{light}");
    }
    let zones = [Zone::Hot, Zone::Habitable, Zone::Cold].map(zone);
    assert_eq!(
        zones,
        [
            "too hot for water",
            "in the habitable zone",
            "too cold for water"
        ]
    );
}

/// A tilt in radians reads in whole degrees.
#[test]
fn tilts_read_in_degrees() {
    let cases = [
        (0.0, "0 deg"),
        (0.409, "23 deg"),
        (PI / 2.0, "90 deg"),
        (PI, "180 deg"),
        (0.0262, "2 deg"),
        (-0.409, "-23 deg"),
    ];
    for (radians, shown) in cases {
        assert_eq!(tilt(radians), shown, "{radians}");
    }
}

/// A year reads in minutes, then hours and minutes, then whole days from two days up.
#[test]
fn years_read_in_play_time_then_days() {
    let cases = [
        (0.0, "0 min"),
        (1500.0, "25 min"),
        (12_000.0, "3 h 20 min"),
        (LONG_YEAR - 1.0, "47 h 59 min"),
        (LONG_YEAR, "2 days"),
        (300_000.0, "3 days"),
        (8.64e6, "100 days"),
    ];
    for (seconds, shown) in cases {
        assert_eq!(year(seconds), shown, "{seconds}");
    }
}
