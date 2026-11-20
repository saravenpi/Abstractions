//! Climate field tests: the Earth check on the home planet, height, seasons, rain, wind, moons and the
//! guarantees every caller leans on (same output, never NaN).

use bevy::math::{DVec3, Vec2};

use super::*;
use crate::universe::{EARTH_RADIUS, PlanetRecord, STANDARD_GRAVITY};

/// Returns the setting of an Earth-like world: the temperate record, a 24 hour day, standard gravity and
/// the Earth's radius. The chosen home differs from universe to universe, so the physics is checked here.
pub(super) fn homeworld() -> Setting {
    let record = PlanetRecord::TEMPERATE;
    Setting {
        temperature: record.temperature as f32,
        pressure: record.pressure as f32,
        ocean: record.ocean as f32,
        tilt: record.tilt as f32,
        day_seconds: 86_400.0,
        gravity: STANDARD_GRAVITY as f32,
        radius: EARTH_RADIUS as f32,
        seed: 42,
        near: None,
    }
}

/// Returns a coastal spot at sea level, flat, at `lat` and `lon` degrees.
pub(super) fn spot(lat: f64, lon: f64) -> Spot {
    let (lat, lon) = (lat.to_radians(), lon.to_radians());
    Spot {
        dir: DVec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin()),
        altitude: 0.0,
        inland: 0.0,
        slope: Vec2::ZERO,
    }
}

/// Returns the mean temperature around the parallel at `lat` degrees, at `season`.
fn ring(setting: &Setting, lat: f64, season: f32) -> f32 {
    (0..72)
        .map(|k| at(setting, &spot(lat, f64::from(k) * 5.0), season).temperature)
        .sum::<f32>()
        / 72.0
}

/// The Earth-like world's equator is about 300 K and its poles about 250 K between the seasons, with its own
/// tilt of about 23 degrees and with none.
#[test]
fn home_has_a_warm_equator_and_cold_poles() {
    let home = homeworld();
    assert_eq!((home.temperature, home.pressure), (288.0, 1.0));
    assert!(
        (home.tilt.to_degrees() - 23.4).abs() < 0.1,
        "tilt {}",
        home.tilt
    );
    for tilt in [home.tilt, 0.0] {
        let setting = Setting { tilt, ..home };
        let (equator, north, south) = (
            ring(&setting, 0.0, 0.25),
            ring(&setting, 89.0, 0.25),
            ring(&setting, -89.0, 0.25),
        );
        assert!((equator - 302.0).abs() < 5.0, "equator {equator}");
        assert!(
            (north - 254.0).abs() < 6.0 && (south - 254.0).abs() < 6.0,
            "poles {north} {south}"
        );
    }
}

/// The equal-area mean of the temperature over the whole sphere is the record mean.
#[test]
fn the_planet_mean_is_the_record_mean() {
    let home = homeworld();
    let n = 4000;
    let total: f32 = (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / f64::from(n);
            spot(y.asin().to_degrees(), f64::from(i) * 137.508)
        })
        .map(|s| at(&home, &s, 0.25).temperature)
        .sum();
    assert!(
        (total / n as f32 - 288.0).abs() < 0.5,
        "mean {}",
        total / n as f32
    );
}

/// Every metre up is colder, at 6.5 K per km at home gravity and twice that at twice the gravity.
#[test]
fn temperature_falls_with_height() {
    let home = homeworld();
    let heavy = Setting {
        gravity: home.gravity * 2.0,
        ..home
    };
    let up = |s: &Setting, altitude: f32| {
        let place = Spot {
            altitude,
            ..spot(40.0, 10.0)
        };
        at(s, &place, 0.1).temperature
    };
    let temps: Vec<f32> = (0..20).map(|k| up(&home, k as f32 * 100.0)).collect();
    assert!(temps.windows(2).all(|w| w[1] < w[0]));
    assert!((temps[0] - temps[10] - 6.5).abs() < 1e-3);
    assert!((up(&heavy, 0.0) - up(&heavy, 1000.0) - 13.0).abs() < 1e-3);
    assert_eq!(up(&home, -300.0), up(&home, 0.0));
}

/// At the home planet's own tilt the north has its summer when the south has its winter, and half a year
/// later the other way round; without a tilt there are no seasons.
#[test]
fn seasons_are_opposite_in_the_two_hemispheres() {
    let tilted = homeworld();
    let swing = |lat: f64, season: f32| ring(&tilted, lat, season) - ring(&tilted, lat, 0.25);
    assert!(swing(50.0, 0.0) > 5.0 && swing(-50.0, 0.0) < -5.0);
    assert!(swing(50.0, 0.5) < -5.0 && swing(-50.0, 0.5) > 5.0);
    assert!((swing(50.0, 0.0) + swing(-50.0, 0.0)).abs() < 0.5);
    let flat = Setting {
        tilt: 0.0,
        ..homeworld()
    };
    assert!((ring(&flat, 50.0, 0.0) - ring(&flat, 50.0, 0.5)).abs() < 1e-3);
}

/// The same inputs always give the same climate, and another seed moves the warped belts.
#[test]
fn same_inputs_give_the_same_climate() {
    let home = homeworld();
    let place = Spot {
        altitude: 40.0,
        inland: 0.3,
        slope: Vec2::new(0.1, -0.2),
        ..spot(31.0, 12.0)
    };
    assert_eq!(at(&home, &place, 0.3), at(&home, &place, 0.3));
    let other = Setting {
        seed: home.seed ^ 1,
        ..home
    };
    let sum = |s: &Setting| {
        (0..40)
            .map(|k| at(s, &spot(28.0, f64::from(k) * 9.0), 0.3).moisture)
            .sum::<f32>()
    };
    assert_ne!(sum(&home), sum(&other));
}
