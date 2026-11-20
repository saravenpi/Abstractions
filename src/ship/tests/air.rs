//! The ship in the air of other planets: thin, thick and none, and a world without a sea.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::{above_pole, coast, fly, planet};
use crate::ship::body::Ship;
use crate::ship::drag::{DRAG, SEA_DENSITY, density, slow};
use crate::ship::flight::{Controls, Report};
use crate::universe::{PlanetRecord, PlanetSpec, heights};
use crate::wind::{PRESSURES, samples};

/// Returns a 40 km planet with an 80 km atmosphere and a very long day, with `record`.
pub(super) fn world(record: PlanetRecord) -> PlanetSpec {
    PlanetSpec {
        record,
        ..planet(40_000.0, 80_000.0, 1e9)
    }
}

/// Returns the home record with another air pressure.
pub(super) fn air(pressure: f64) -> PlanetRecord {
    PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    }
}

/// The density is the home density times the pressure: the same number as before at home, at
/// any height, nothing without air and never less as the pressure grows.
#[test]
fn density_follows_the_pressure() {
    let home = planet(40_000.0, 80_000.0, 1e9);
    assert_eq!(density(&home, 0.0), SEA_DENSITY);
    for height in [0.0, 120.0, 2500.0, 40_000.0, 79_999.0, 80_000.0] {
        let seen = PRESSURES.map(|at| density(&world(air(at)), height));
        assert_eq!(seen[3], density(&home, height));
        assert_eq!((seen[0], seen[1]), (0.0, 0.0));
        assert!(seen.windows(2).all(|pair| pair[0] <= pair[1]), "{seen:?}");
        assert!((seen[2] - 0.3 * seen[3]).abs() < 1e-12 && seen[4] <= 3.0 * seen[3]);
    }
    assert_eq!(density(&world(air(50.0)), 0.0), 3.0 * SEA_DENSITY);
}

/// Returns the speed and the height of a nose-down fall from `top` metres over `planet`, as it
/// passes 1 km, and the highest dynamic pressure reported on the way.
fn dive(planet: &PlanetSpec, top: f64) -> (f64, f64, f64) {
    let mut ship = above_pole(planet, top);
    ship.rot = DQuat::from_rotation_x(-TAU / 4.0);
    let mut seen = (0.0, 0.0, 0.0f64);
    let watch = |ship: &Ship, report: &Report| {
        let height = ship.pos.length() - planet.radius;
        if height >= 1000.0 {
            seen = (ship.vel.length(), height, seen.2.max(report.pressure));
        }
    };
    let land = (planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    fly(&mut ship, &Controls::default(), land, 240.0, watch);
    seen
}

/// In thin air the ship falls faster and in thick air slower, each at the speed where the drag
/// of that air equals the weight. The atmosphere is 1,200 km deep, a scale height of 100 km, so
/// that the air hardly changes over the distance the thin air needs to settle the speed.
#[test]
fn terminal_speed_follows_the_air() {
    let speeds = [0.3, 1.0, 3.0].map(|at| {
        let planet = PlanetSpec {
            atmosphere_height: 1_200_000.0,
            ..world(air(at))
        };
        let (speed, height, _) = dive(&planet, 12_000.0);
        let pull = planet.mu() / (planet.radius + height).powi(2);
        let expected = (pull / (DRAG.z * density(&planet, height))).sqrt();
        let off = (speed / expected - 1.0).abs();
        assert!(off < 0.05, "{speed} against {expected} at {at}");
        speed
    });
    let ordered = speeds[0] > 1.5 * speeds[1] && speeds[1] > 1.5 * speeds[2];
    assert!(ordered, "{speeds:?}");
}

/// Over a planet without air the ship falls as in space: the speed a fall in vacuum gives, no
/// dynamic pressure, so no glow, no shake and no rumble, and no air turns the hull.
#[test]
fn an_airless_planet_has_no_air_forces() {
    for record in [air(0.0), air(0.005), samples()[5]] {
        let planet = world(record);
        let (speed, height, pressure) = dive(&planet, 4000.0);
        let (from, to) = (planet.radius + 4000.0, planet.radius + height);
        let vacuum = (2.0 * planet.mu() * (1.0 / to - 1.0 / from)).sqrt();
        assert!((speed / vacuum - 1.0).abs() < 1e-3, "{speed} for {vacuum}");
        assert_eq!(pressure, 0.0);
        let mut ship = above_pole(&planet, 4000.0);
        ship.vel = DVec3::X * 30.0;
        coast(&mut ship, &Controls::default(), (&planet, &|_| 0.0), 5.0);
        assert_eq!((ship.rot, ship.spin), (DQuat::IDENTITY, DVec3::ZERO));
        assert!((ship.vel.x - 30.0).abs() < 0.5, "slowed to {}", ship.vel.x);
    }
}

/// At the home pressure the class changes nothing in the air: the same flight, bit for bit,
/// over all six kinds of world.
#[test]
fn the_class_alone_changes_no_flight() {
    let flown = samples().map(|record| {
        let planet = world(PlanetRecord {
            pressure: 1.0,
            ..record
        });
        let mut ship = above_pole(&planet, 900.0);
        (ship.vel, ship.throttle) = (DVec3::new(40.0, -20.0, -90.0), 0.7);
        coast(&mut ship, &Controls::default(), (&planet, &|_| 300.0), 8.0);
        (ship.pos, ship.vel, ship.rot, ship.spin)
    });
    assert!(flown.iter().all(|ship| *ship == flown[1]));
}

/// On a planet with the size, the pull and the air of the Earth the scale height is 8.4 km: the
/// air at the sea is e times denser than 8.5 km up, it drags a ship e times harder there, and
/// there is none left at the top.
#[test]
fn the_air_of_an_earth_thins_by_e_over_its_scale_height() {
    let (radius, gravity) = (6.371e6, 9.81);
    let [atmosphere_height, cloud_base, cloud_top] =
        heights(&PlanetRecord::TEMPERATE, gravity, radius);
    let earth = PlanetSpec {
        gravity,
        atmosphere_height,
        cloud_base,
        cloud_top,
        ..planet(radius, 0.0, 86_164.0)
    };
    assert!((7000.0..10_000.0).contains(&earth.scale_height()));
    assert_eq!(density(&earth, 0.0), SEA_DENSITY);
    let vel = DVec3::NEG_Z * 200.0;
    let drag = |height: f64| {
        let after = slow(DQuat::IDENTITY, vel, density(&earth, height), 0.01);
        (vel - after).length()
    };
    for fall in [
        density(&earth, 0.0) / density(&earth, 8500.0),
        drag(0.0) / drag(8500.0),
    ] {
        assert!((fall / std::f64::consts::E - 1.0).abs() < 0.03, "{fall}");
    }
    assert_eq!(density(&earth, atmosphere_height), 0.0);
    assert!(density(&earth, atmosphere_height - 1.0) > 0.0);
}
