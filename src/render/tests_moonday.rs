//! Day tests: the day from noon to noon follows from the physical spin and the orbit alone. The
//! star goes round a moon's sky once per solar day of its planet's year and its own turn, and
//! asking for a time of day moves universe time forward to reach it.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use super::clock::{catch_up, solar_day};
use super::observer::Observer;
use crate::space::{Anchor, Frame};
use crate::universe::{PlanetSpec, StarSpec, Universe};

/// Returns the innermost moon of the first planet with moons near the home of seed 3, and the
/// star.
fn moon() -> (PlanetSpec, StarSpec) {
    let universe = Universe::new(3);
    let home = universe.home().star;
    let at = universe.star(home).expect("the home star exists").pos;
    let near = universe.stars_near(home.galaxy, at, 60.0);
    let mooned = near.iter().find_map(|star| {
        let planets = universe.system(star.id).planets;
        let host = planets.into_iter().find(|planet| planet.moons > 0)?;
        Some((universe.moons(host.id).remove(0), star.clone()))
    });
    mooned.expect("a planet near home has a moon")
}

/// Returns a surface anchor standing at sea level where the moon-fixed direction `up` leaves it.
fn standing(moon: &PlanetSpec, up: DVec3) -> Anchor {
    let east = DVec3::Y.cross(up).normalize();
    let to_frame = DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up)));
    Anchor {
        frame: Frame::Surface(moon.id),
        origin: up * moon.radius,
        rotation: to_frame.inverse(),
        flat: false,
    }
}

/// The solar day of a moon is finite, the phase goes round once in it, and a wanted phase is
/// reached by moving universe time forward, whichever way the star crosses the moon's sky.
#[test]
fn a_moon_has_days_and_reaches_a_wanted_time() {
    let (moon, star) = moon();
    let anchor = standing(&moon, DVec3::new(0.3, 0.5, -0.8).normalize());
    let phase = |time: f64| Observer::new(&anchor, Vec3::ZERO, &moon, &star, time).phase();
    let day = solar_day(&moon);
    assert!(day.is_finite() && day.abs() > 1.0, "{day}");
    let start = 5_000.0;
    let turned = (phase(start + day.abs()) - phase(start) + 0.5).rem_euclid(1.0) - 0.5;
    assert!(turned.abs() < 2e-3, "{turned}");
    for wanted in [0.0, 0.08, 0.25, 0.6, 0.99] {
        let later = start + catch_up(phase(start), wanted, &moon);
        let off = (phase(later) - wanted + 0.5).rem_euclid(1.0) - 0.5;
        assert!(
            later >= start && off.abs() < 1e-3,
            "{wanted}: {}",
            phase(later)
        );
    }
}

/// A planet that turns once in 86,164 s and goes round its star in a year has the day of the
/// Earth: 86,400 s from noon to noon, though nothing sets that length.
#[test]
fn an_earth_twin_has_a_day_of_24_hours() {
    let (moon, _) = moon();
    let twin = PlanetSpec {
        day_seconds: 86_164.09,
        orbit_seconds: 365.256_36 * 86_400.0,
        host: None,
        ..moon
    };
    let day = solar_day(&twin);
    assert!((day - 86_400.0).abs() < 60.0, "{day}");
    assert!(!twin.locked());
    assert!((catch_up(0.1, 0.6, &twin) - 0.5 * day).abs() < 1.0);
}

/// A tidally locked moon turns once per orbit around its planet, so its day from noon to noon
/// is that orbit, a little longer by the share of its planet's year that goes by meanwhile.
#[test]
fn a_locked_moon_has_the_day_of_its_orbit() {
    let (moon, _) = moon();
    let year = moon.host.expect("a moon has a host").seconds;
    assert!(moon.locked());
    assert_eq!(moon.day_seconds, moon.orbit_seconds);
    let day = solar_day(&moon);
    let stretch = 1.0 / (1.0 - moon.orbit_seconds / year);
    assert!(
        (day / moon.orbit_seconds / stretch - 1.0).abs() < 1e-9,
        "{day}"
    );
    assert!(day >= moon.orbit_seconds);
}

/// A planet locked to its star has no day: the star stands still in its sky, and asking for
/// another time of day moves no time.
#[test]
fn a_locked_planet_has_no_day() {
    let (moon, _) = moon();
    let locked = PlanetSpec {
        day_seconds: 3.0e6,
        orbit_seconds: 3.0e6,
        host: None,
        ..moon
    };
    assert!(locked.locked() && solar_day(&locked).is_infinite());
    assert_eq!(catch_up(0.1, 0.6, &locked), 0.0);
    let slow = PlanetSpec {
        day_seconds: 4.0e6,
        ..locked.clone()
    };
    assert!(solar_day(&slow) < 0.0);
    assert!(catch_up(0.1, 0.6, &slow) > 0.0);
}
