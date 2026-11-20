//! Tests of the heat of an entry: none without air, eight times the heat for twice the speed,
//! thick air glowing earlier than thin air, a colour for each gas, and a glow that eases and
//! costs nothing while the craft is slow.

use super::{Craft, Gas, Glow, Heat, descent, entry, flux};
use crate::universe::{self as air, Mix};

/// Density of the air of the home planet at sea level in kilograms per cubic metre.
const HOME: f64 = 1.2;

/// Density at sea level of a planet with a tenth of the home air.
const THIN: f64 = 0.12;

/// Density at sea level of a planet with three times the home air.
const THICK: f64 = 3.6;

/// Without air nothing glows and nothing presses, at any speed.
#[test]
fn no_air_gives_no_glow() {
    let glow = entry(0.0, 5000.0, Gas::NitrogenOxygen);
    assert_eq!((glow.heat, glow.pressure), (0.0, 0.0));
    assert_eq!(flux(0.0, 5000.0), 0.0);
}

/// Twice the speed in the same air gives eight times the heat flux, and four times the density
/// gives twice.
#[test]
fn the_flux_follows_the_cube_of_the_speed() {
    let slow = flux(HOME, 100.0);
    assert!((flux(HOME, 200.0) / slow - 8.0).abs() < 1.0e-9);
    assert!((flux(4.0 * HOME, 100.0) / slow - 2.0).abs() < 1.0e-9);
}

/// At the same speed thick air glows harder than home air and home air harder than thin air, until
/// both glow in full, and a slow craft glows in none of them.
#[test]
fn thick_air_glows_earlier_than_thin() {
    let at = |density: f64, speed: f64| entry(density, speed, Gas::NitrogenOxygen).heat;
    for speed in [120.0, 160.0] {
        assert!(at(THICK, speed) > at(HOME, speed), "{speed}");
        assert!(at(HOME, speed) > at(THIN, speed), "{speed}");
    }
    assert_eq!((at(THICK, 240.0), at(HOME, 240.0)), (1.0, 1.0));
    assert!(at(HOME, 240.0) > at(THIN, 240.0));
    for density in [THIN, HOME, THICK] {
        assert_eq!(at(density, 20.0), 0.0, "{density}");
    }
    assert!(at(THIN, 240.0) < 0.6 && at(THICK, 240.0) > 0.95);
}

/// Every gas glows in a colour of its own, and a glow that heats up changes its colour.
#[test]
fn colours_differ_by_gas() {
    let gases = [
        Gas::NitrogenOxygen,
        Gas::CarbonDioxide,
        Gas::Hydrogen,
        Gas::Methane,
        Gas::Sulphur,
        Gas::Unknown,
    ];
    for (i, one) in gases.iter().enumerate() {
        for other in &gases[i + 1..] {
            assert_ne!(one.color(0.4), other.color(0.4), "{one:?} {other:?}");
        }
        assert_ne!(one.color(0.1), one.color(0.9), "{one:?}");
    }
}

/// Level flight and a landing approach show nothing, a dive shows all of it.
#[test]
fn only_a_fall_counts_as_an_entry() {
    assert_eq!(descent(0.0), 0.0);
    assert_eq!(descent(18.0), 0.0);
    assert_eq!(descent(110.0), 1.0);
    let glow = entry(HOME, 240.0, Gas::Unknown);
    assert_eq!(glow.scaled(0.0).heat, 0.0);
    assert_eq!(glow.scaled(1.0), glow);
}

/// The glow rises and cools at bounded rates and is exactly zero once it has cooled.
#[test]
fn the_glow_eases_and_cools() {
    let hot = entry(THICK, 240.0, Gas::CarbonDioxide);
    let rising = Glow::NONE.follow(hot, 0.1);
    assert!(rising.heat > 0.0 && rising.heat < hot.heat);
    assert_eq!(rising.color, hot.color);
    let mut cooling = hot;
    for _ in 0..2000 {
        cooling = cooling.follow(Glow::NONE, 0.05);
    }
    assert_eq!(cooling.heat, 0.0);
}

/// Without an offer the heat does no work, and an offer wakes it.
#[test]
fn an_idle_heat_does_no_work() {
    let mut heat = Heat::default();
    assert!(!heat.live());
    heat.offer(entry(HOME, 240.0, Gas::NitrogenOxygen), Craft::default());
    assert!(heat.live());
}

/// The most abundant gas of a mix gives the gas of the glow.
#[test]
fn a_mix_gives_a_gas() {
    assert_eq!(Gas::of(&Mix::HOME), Gas::NitrogenOxygen);
    let giant = Mix::of([
        (air::Gas::Hydrogen, 0.9),
        (air::Gas::Helium, 0.1),
        (air::Gas::Hydrogen, 0.0),
        (air::Gas::Hydrogen, 0.0),
    ]);
    assert_eq!(Gas::of(&giant), Gas::Hydrogen);
    let rock = |gas, rest| Mix::of([(gas, 0.9), (rest, 0.1), (rest, 0.0), (rest, 0.0)]);
    let nitrogen = air::Gas::Nitrogen;
    assert_eq!(
        Gas::of(&rock(air::Gas::CarbonDioxide, nitrogen)),
        Gas::CarbonDioxide
    );
    assert_eq!(
        Gas::of(&rock(air::Gas::SulphurDioxide, nitrogen)),
        Gas::Sulphur
    );
    assert_eq!(Gas::of(&rock(air::Gas::Methane, nitrogen)), Gas::Methane);
}
