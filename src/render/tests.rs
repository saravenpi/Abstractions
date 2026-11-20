//! Sky tests: the sun and the day phase follow the real star, the air and the clouds follow the
//! height, and far bodies are drawn where they truly are.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use super::atmosphere::{air, horizon_dip, shadow};
use super::bodies::{FAR, place};
use super::clock::{catch_up, solar_day};
use super::clouds::crossing_depth;
use super::fog::{Height, fog_sight};
use super::observer::Observer;
use super::sun::SunDir;
use crate::space::{Anchor, Frame};
use crate::universe::{PlanetSpec, StarSpec, Universe};
use crate::world::Terrain;

/// Returns the home planet and its star for a seed. The planet is made to turn 300 times a
/// year, so that it has days whatever tides did to the one the seed chose.
fn home(seed: u64) -> (PlanetSpec, StarSpec) {
    let universe = Universe::new(seed);
    let body = universe.home();
    let planet = universe.planet(body);
    let turning = PlanetSpec {
        day_seconds: planet.orbit_seconds / 300.0,
        ..planet
    };
    (turning, universe.system(body.star).star)
}

/// Returns a surface anchor standing at sea level where the planet-fixed direction `up` leaves
/// the planet, with render +Y up.
fn standing(planet: &PlanetSpec, up: DVec3) -> Anchor {
    let east = DVec3::Y.cross(up).normalize();
    let to_frame = DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up)));
    Anchor {
        frame: Frame::Surface(planet.id),
        origin: up * planet.radius,
        rotation: to_frame.inverse(),
        flat: false,
    }
}

/// Returns the planet-fixed direction of the spot that faces the star at `time`.
fn facing(planet: &PlanetSpec, star: &StarSpec, time: f64) -> DVec3 {
    let anywhere = standing(planet, DVec3::X);
    let observer = Observer::new(&anywhere, Vec3::ZERO, planet, star, time);
    observer.spin.inverse() * (-observer.center).normalize()
}

/// The sun is high and the phase is noon where the planet faces its star, midnight on the other
/// side, and the legacy flat world still starts its day with the sun rising along +X.
#[test]
fn noon_is_where_the_planet_faces_the_star() {
    let (planet, star) = home(7);
    for time in [0.0, 4321.0, 987_654.0] {
        let toward = facing(&planet, &star, time);
        let day = Observer::new(&standing(&planet, toward), Vec3::ZERO, &planet, &star, time);
        let sun = SunDir::of(&day, planet.radius);
        assert!(sun.height > 0.99 && sun.dir.y > 0.99, "{time} {sun:?}");
        assert!((day.phase() - 0.25).abs() < 0.01, "{time} {}", day.phase());
        assert!((day.altitude(planet.radius)).abs() < 1e-6 && sun.lit == 1.0);
        let night = Observer::new(
            &standing(&planet, -toward),
            Vec3::ZERO,
            &planet,
            &star,
            time,
        );
        assert!(SunDir::of(&night, planet.radius).height < -0.99);
        assert!(
            (night.phase() - 0.75).abs() < 0.01,
            "{time} {}",
            night.phase()
        );
    }
    let terrain = Terrain::new(7);
    let flat = Anchor::surface(planet.id, &terrain, Vec3::new(40.0, 12.0, -90.0));
    let mut time = 0.0;
    let at = |time: f64| Observer::new(&flat, Vec3::new(40.0, 12.0, -90.0), &planet, &star, time);
    time += catch_up(at(time).phase(), 0.0, &planet);
    let rise = SunDir::of(&at(time), planet.radius);
    assert!(rise.dir.x > 0.99 && rise.height.abs() < 0.02, "{rise:?}");
    assert!((at(time).altitude(planet.radius) - 12.0).abs() < 1e-6);
}

/// The phase only moves forward through a day, covers the whole cycle in one solar day, and a
/// wanted phase is reached by moving universe time forward.
#[test]
fn phase_rises_through_a_day() {
    let (planet, star) = home(3);
    let anchor = standing(&planet, DVec3::new(0.3, 0.5, -0.8).normalize());
    let phase = |time: f64| Observer::new(&anchor, Vec3::ZERO, &planet, &star, time).phase();
    let (start, day, steps) = (5_000.0, solar_day(&planet), 400);
    let mut total = 0.0;
    for step in 0..steps {
        let now = phase(start + day * f64::from(step) / f64::from(steps));
        let next = phase(start + day * f64::from(step + 1) / f64::from(steps));
        let gain = (next - now).rem_euclid(1.0);
        assert!(gain > 0.0 && gain < 0.01, "step {step}: {now} -> {next}");
        total += gain;
    }
    assert!((total - 1.0).abs() < 1e-3, "{total}");
    for wanted in [0.0, 0.08, 0.25, 0.6, 0.99] {
        let later = start + catch_up(phase(start), wanted, &planet);
        let off = (phase(later) - wanted + 0.5).rem_euclid(1.0) - 0.5;
        assert!(
            later >= start && off.abs() < 1e-4,
            "{wanted}: {}",
            phase(later)
        );
        assert!(catch_up(phase(later), wanted, &planet) < 1.0);
    }
}

/// The air is whole at the ground and gone at the top of the atmosphere, the horizon drops and
/// the fog reaches farther with height, and the planet casts a shadow behind it only.
#[test]
fn air_thins_with_height() {
    let height = 4000.0;
    assert_eq!(air(0.0, height), 1.0);
    assert_eq!(air(-20.0, height), 1.0);
    assert_eq!(air(height, height), 0.0);
    assert_eq!(air(9.0 * height, height), 0.0);
    assert_eq!(air(10.0, 0.0), 0.0);
    let levels: Vec<f32> = (0..=40).map(|i| air(i as f32 * 100.0, height)).collect();
    assert!(levels.windows(2).all(|pair| pair[1] < pair[0]));
    assert_eq!(horizon_dip(0.0, 6000.0), 0.0);
    assert!((horizon_dip(4000.0, 6000.0) - 0.8).abs() < 1e-5);
    let space = Height {
        altitude: 4200.0,
        clearance: 4000.0,
        radius: 3000.0,
        air: air(4200.0, height),
    };
    assert!(fog_sight(256.0, &space) > 1.0e7);
    let (sun, radius) = (DVec3::X, 6000.0);
    assert_eq!(shadow(DVec3::new(7000.0, 0.0, 0.0), sun, radius), 1.0);
    assert_eq!(shadow(DVec3::new(-7000.0, 100.0, 0.0), sun, radius), 0.0);
    assert_eq!(shadow(DVec3::new(-7000.0, 6500.0, 0.0), sun, radius), 1.0);
}

/// The crossing depth is zero below the base and above the top and one in the middle of the band.
#[test]
fn cloud_crossing_peaks_mid_band() {
    let (base, top) = (900.0, 1500.0);
    for outside in [-50.0, 0.0, 899.0, 900.0, 1500.0, 1501.0, 9000.0] {
        assert_eq!(crossing_depth(outside, base, top), 0.0, "{outside}");
    }
    assert_eq!(crossing_depth(1200.0, base, top), 1.0);
    let rising: Vec<f32> = (0..=30)
        .map(|i| crossing_depth(900.0 + i as f32 * 10.0, base, top))
        .collect();
    assert!(rising.windows(2).all(|pair| pair[1] > pair[0]));
    assert!((crossing_depth(1000.0, base, top) - crossing_depth(1400.0, base, top)).abs() < 1e-5);
    assert_eq!(crossing_depth(1200.0, 1500.0, 1500.0), 0.0);
}

/// A far body is drawn in its true direction, covers its true angle, sits within reach of 32-bit
/// positions and stays behind every nearer body.
#[test]
fn far_bodies_keep_direction_and_size() {
    let toward = DVec3::new(0.36, -0.48, 0.8);
    let radius = 6000.0;
    let mut last = 0.0;
    for distance in [10.0, 9.0e5, 1.0e6, 1.2e6, 4.0e7, 5.0e8, 3.0e11, 1.0e16] {
        let drawn = place(toward * distance);
        let reach = drawn.at.length();
        assert!(
            (drawn.at / reach - toward.as_vec3()).length() < 1e-5,
            "{distance}"
        );
        let (seen, truth) = (
            radius * f64::from(drawn.scale) / f64::from(reach),
            radius / distance,
        );
        assert!(
            (seen / truth - 1.0).abs() < 0.01,
            "{distance}: {seen} {truth}"
        );
        assert!(reach > last && reach <= 2.0 * FAR, "{distance}: {reach}");
        assert!(distance > f64::from(FAR) || drawn.scale == 1.0);
        last = reach;
    }
}
