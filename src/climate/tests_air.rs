//! Climate field tests for wetness, wind and moons, and the guarantee that no record gives NaN.

use bevy::math::{DVec3, Vec2, Vec3};

use super::tests::{homeworld, spot};
use super::*;
use crate::universe::{PlanetSpec, Universe};

/// Returns the first moon of the home systems of the first universes, with its planet.
fn a_moon() -> (PlanetSpec, PlanetSpec) {
    (0..64u64)
        .flat_map(|seed| {
            let universe = Universe::new(seed);
            let planets = universe.system(universe.home().star).planets;
            let hosts = planets.into_iter().filter(|p| p.moons > 0);
            hosts
                .flat_map(|p| {
                    universe
                        .moons(p.id)
                        .into_iter()
                        .map(move |m| (m, p.clone()))
                })
                .collect::<Vec<_>>()
        })
        .next()
        .expect("a moon")
}

/// The wetness falls inland, is low in the subtropical belt, high at the equator and zero on a dry world.
#[test]
fn moisture_falls_inland_and_in_the_dry_belts() {
    let home = homeworld();
    let wet = |inland: f32, lat: f64| {
        at(
            &home,
            &Spot {
                inland,
                ..spot(lat, 30.0)
            },
            0.0,
        )
        .moisture
    };
    assert!(wet(1.0, 50.0) < wet(0.5, 50.0) && wet(0.5, 50.0) < wet(0.0, 50.0));
    assert_eq!(wet(-1.0, 50.0), wet(0.0, 50.0));
    assert!(wet(0.0, 25.0) < 0.3 * wet(0.0, 2.0) && wet(0.0, 25.0) < 0.4 * wet(0.0, 50.0));
    let dry = Setting { ocean: 0.0, ..home };
    assert_eq!(at(&dry, &spot(2.0, 0.0), 0.0).moisture, 0.0);
    let slow = Setting {
        day_seconds: 6.0 * 86_400.0,
        ..home
    };
    let belt = |s: &Setting| at(s, &spot(50.0, 30.0), 0.0).moisture;
    assert!(belt(&slow) < 0.4 * belt(&home));
}

/// A slope that rises into the wind is wetter than flat ground, a lee slope drier, both capped.
#[test]
fn windward_slopes_catch_the_rain() {
    let home = homeworld();
    let flat = spot(15.0, 0.0);
    let wind = at(&home, &flat, 0.0).wind.normalize();
    let on = |slope: Vec2| at(&home, &Spot { slope, ..flat }, 0.0).moisture;
    let base = on(Vec2::ZERO);
    assert!(on(wind * 0.3) > 1.4 * base && on(-wind * 0.3) < 0.6 * base);
    assert!(
        (on(wind * 50.0) - 2.5 * base).abs() < 1e-4 && (on(-wind * 50.0) - 0.3 * base).abs() < 1e-4
    );
}

/// Trades blow from the east toward the equator, westerlies from the west toward the pole,
/// polar winds from the east, mirrored in the south; no air gives no wind.
#[test]
fn wind_bands_follow_the_latitude() {
    let home = homeworld();
    let wind = |s: &Setting, lat: f64| at(s, &spot(lat, 70.0), 0.0).wind;
    let (trade, west, polar) = (wind(&home, 15.0), wind(&home, 45.0), wind(&home, 75.0));
    assert!(trade.x < 0.0 && trade.y < 0.0 && west.x > 0.0 && west.y > 0.0 && polar.x < 0.0);
    assert!(wind(&home, -15.0).y > 0.0 && wind(&home, -45.0).y < 0.0);
    assert!((4.0..=12.01).contains(&polar.length()) && (11.9..=12.01).contains(&west.length()));
    let thick = Setting {
        pressure: 4.0,
        ..home
    };
    assert!((wind(&thick, 45.0).length() - 2.0 * west.length()).abs() < 1e-3);
    let airless = Setting {
        pressure: 0.0,
        ..home
    };
    assert!((-80..=80).all(|lat| wind(&airless, f64::from(lat)) == Vec2::ZERO));
}

/// A tidally locked moon is warm under its planet and cold on its far side, and `Setting::of`
/// puts the warm side where the planet really is in the moon's frame.
#[test]
fn a_moon_is_warm_on_its_near_side() {
    let (moon, _) = a_moon();
    let setting = Setting::of(&moon);
    let near = setting.near.expect("a locked moon");
    let time = 1234.5;
    let planet_at = moon.host.expect("a host").at(time);
    let toward =
        crate::world::spin(moon.day_seconds, time).inverse() * (planet_at - moon.position(time));
    assert!(toward.normalize().as_vec3().distance(near) < 1e-4);
    let face = |d: Vec3| {
        at(
            &setting,
            &Spot {
                dir: d.as_dvec3(),
                ..spot(0.0, 0.0)
            },
            0.25,
        )
        .temperature
    };
    assert!((face(near) - setting.temperature - 15.0).abs() < 1.0);
    assert!((face(-near) - setting.temperature + 15.0).abs() < 1.0);
}

/// Returns a hand-built setting for each class: ocean, island, desert, ice, lava, barren, a gas
/// giant, a close locked moon and a far one, with the extremes of every range.
fn records() -> [Setting; 9] {
    let home = homeworld();
    let near = Some(Vec3::new(0.6, 0.0, -0.8));
    let rows = [
        (288.0, 0.95, 1.4, 0.4, 1200.0, 22.0),
        (288.0, 0.6, 1.0, 0.41, 1200.0, 22.0),
        (330.0, 0.0, 0.6, 0.4, 600.0, 10.0),
        (180.0, 0.0, 0.3, 1.57, 1200.0, 22.0),
        (1500.0, 0.5, 5.0, 0.1, 3.0 * 86_400.0, 40.0),
        (40.0, 0.0, 0.0, 0.0, 900.0, 2.0),
        (120.0, 0.0, 1000.0, 0.05, 2000.0, 60.0),
        (260.0, 0.6, 0.005, 0.3, -800.0, 3.0),
        (90.0, 0.2, 1.0, 0.2, -220_000.0, 1.0),
    ];
    rows.map(
        |(temperature, ocean, pressure, tilt, day_seconds, gravity)| Setting {
            temperature,
            ocean,
            pressure,
            tilt,
            day_seconds,
            gravity,
            near: (day_seconds < 0.0).then_some(near).flatten(),
            ..home
        },
    )
}

/// No record, place or season gives a temperature, wetness or wind that is not a finite number.
#[test]
fn no_record_gives_nan() {
    for setting in records() {
        for i in 0..2000 {
            let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / 2000.0;
            let place = Spot {
                altitude: (i % 13) as f32 * 600.0 - 900.0,
                inland: (i % 7) as f32 / 3.0 - 1.0,
                slope: Vec2::new((i % 5) as f32 - 2.0, (i % 3) as f32 * 1.5 - 1.5),
                ..spot(
                    y.clamp(-1.0, 1.0).asin().to_degrees(),
                    f64::from(i) * 137.508,
                )
            };
            let c = at(&setting, &place, (i % 17) as f32 / 17.0);
            assert!(
                c.temperature.is_finite() && c.moisture.is_finite() && c.wind.is_finite(),
                "{setting:?} {c:?}"
            );
            assert!(c.moisture >= 0.0);
        }
        let pole = Spot {
            dir: DVec3::Y,
            ..spot(0.0, 0.0)
        };
        assert!(at(&setting, &pole, 0.0).temperature.is_finite());
    }
}

/// The season is the share of the orbit gone round, a quarter more after a quarter orbit, and a
/// moon's season is the season of its planet.
#[test]
fn the_season_follows_the_orbit() {
    let universe = Universe::new(42);
    let home = universe.planet(universe.home());
    let quarter = home.orbit_seconds / 4.0;
    for time in [0.0, 1.0e3, 7.7e6] {
        let (now, later) = (season(&home, time), season(&home, time + quarter));
        assert!((0.0..1.0).contains(&now) && ((later - now).rem_euclid(1.0) - 0.25).abs() < 1e-4);
    }
    let (moon, planet) = a_moon();
    assert!(
        [0.0, 5.0e4, 3.3e7]
            .iter()
            .all(|&t| season(&moon, t) == season(&planet, t))
    );
}
