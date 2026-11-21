//! Star path tests: where the star stands in the frame of a planet, and how high it climbs over a
//! day on a planet that turns and on one locked to its star.

use bevy::math::DVec3;

use super::*;
use crate::climate::{Fair, Setting};

/// Returns the home planet of the world of `seed`.
fn home(seed: u64) -> PlanetSpec {
    let universe = Universe::new(seed);
    universe.planet(universe.home())
}

/// Returns the home planet of seed 7 as it would be locked to its star.
fn locked() -> PlanetSpec {
    let planet = home(7);
    PlanetSpec {
        day_seconds: planet.orbit_seconds,
        ..planet
    }
}

/// Returns the home planet of seed 7 as it would be turning once in 30 hours.
fn turning() -> PlanetSpec {
    PlanetSpec {
        day_seconds: 108_000.0,
        ..home(7)
    }
}

/// Returns the unit direction `degrees` north of the equator, at `east` radians round the spin axis.
fn place(degrees: f64, east: f64) -> DVec3 {
    let lat = degrees.to_radians();
    DVec3::new(lat.cos() * east.cos(), lat.sin(), lat.cos() * east.sin())
}

/// On a planet locked to its star the star never moves in the frame of the planet: it stays where
/// the climate puts the warm side, and it climbs no higher over a day than it stands at any time.
#[test]
fn the_star_of_a_locked_planet_stands_still() {
    let planet = locked();
    assert!(planet.locked() && solar_day(&planet).is_infinite());
    let first = sun(&planet, 0.0);
    for time in [3.0e3, 8.0e5, 4.0e7, 2.0e9] {
        assert!(sun(&planet, time).distance(first) < 1e-6, "at {time} s");
    }
    let near = Setting::of(&planet)
        .near
        .expect("a locked planet has a near side");
    assert!(near.as_dvec3().distance(first) < 1e-4);
    let day = Daystar::of(&planet);
    for dir in [first, -first, DVec3::Y, place(30.0, 1.0), place(-40.0, 4.0)] {
        assert!((day.peak(dir) - dir.dot(first)).abs() < 1e-6, "{dir}");
    }
}

/// On a planet that turns the star goes round the equator once a day, so the highest it climbs at
/// a place is the cosine of the latitude there, at every longitude. A globe built without an orbit
/// has the same day.
#[test]
fn the_star_of_a_turning_planet_climbs_by_the_latitude() {
    let planet = turning();
    assert!(!planet.locked() && solar_day(&planet).is_finite());
    for day in [Daystar::of(&planet), Daystar::default()] {
        for (degrees, east) in [
            (0.0_f64, 0.3_f64),
            (25.0, 2.0),
            (-48.0, 5.1),
            (65.0, 3.3),
            (90.0, 0.0),
        ] {
            let (peak, cos) = (day.peak(place(degrees, east)), degrees.to_radians().cos());
            assert!((peak - cos).abs() < 3e-3, "{degrees} degrees: {peak}");
        }
    }
}

/// The star reaches the height a base needs, [`LIT`], on a turning planet up to 65 degrees of
/// latitude, and on a locked one only within 65 degrees of the point under the star.
#[test]
fn the_lit_ground_is_the_day_side_of_a_locked_planet() {
    let lit = LIT.to_radians().sin();
    let day = Daystar::of(&turning());
    assert!(day.peak(place(60.0, 1.0)) > lit && day.peak(place(70.0, 1.0)) < lit);
    let planet = locked();
    let (day, under) = (Daystar::of(&planet), sun(&planet, 0.0));
    let east = DVec3::Y.cross(under);
    let off = |degrees: f64| under * degrees.to_radians().cos() + east * degrees.to_radians().sin();
    assert!(day.peak(off(60.0)) > lit && day.peak(off(70.0)) < lit);
    assert!(day.peak(off(180.0)) < -0.99);
}

/// Every home has a place in mild air under a high star: on the half circle from the point under
/// the star at time 0 to the pole, some place lacks nothing. On a locked home that place is on the
/// day side, and the home is not too hot there.
#[test]
fn every_home_has_a_fair_place() {
    for seed in 0..40 {
        let planet = home(seed);
        let fair = Fair::of(&Setting::of(&planet), Daystar::of(&planet));
        let under = sun(&planet, 0.0);
        let up = |degrees: f64| {
            under * degrees.to_radians().cos() + DVec3::Y * degrees.to_radians().sin()
        };
        let found = (0..=650).any(|tenth| fair.lack(up(f64::from(tenth) / 10.0)) == 0.0);
        assert!(
            found,
            "seed {seed}: no mild place in daylight, locked {}",
            planet.locked()
        );
    }
}
