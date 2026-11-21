//! The arrival shell: a course aimed at an Earth-size planet, a small moon and a gas giant ends just outside
//! the air at rest beside the body, never inside it or its air under warp, and the flight from there to
//! the air takes seconds to two minutes.

use bevy::math::{DQuat, DVec3};

use super::ascent_tests::earth;
use super::shell::{lock, radius};
use crate::ship::body::Ship;
use crate::ship::cruise::top_speed_at;
use crate::ship::warp::{CRUISE_CAP, Warp, advance, drop_out};
use crate::ship::wells::Well;
use crate::space::Frame;
use crate::universe::PlanetSpec;

/// Seconds of one frame of the flights.
const DT: f64 = 0.05;

/// Seconds the warp may take from far away.
const WARP_LIMIT: f64 = 600.0;

/// Seconds from the drop-out to the top of the air the descent may take.
const DESCENT_LIMIT: f64 = 120.0;

/// Metres from the planet the course starts at, in planet radii.
const START: f64 = 400.0;

/// A small moon with no air.
fn moon() -> PlanetSpec {
    PlanetSpec {
        radius: 1.7e6,
        gravity: 1.6,
        atmosphere_height: 0.0,
        ..earth()
    }
}

/// A gas giant with a thick air.
fn giant() -> PlanetSpec {
    PlanetSpec {
        radius: 7.0e7,
        gravity: 25.0,
        atmosphere_height: 1.5e6,
        ..earth()
    }
}

/// Flies a ship from [`START`] radii out to `planet` in warp, dropping out as the autopilot does when it
/// is within the arrival shell, and returns the well, the ship and the least distance to the centre of the
/// planet it flew at while in warp.
fn warp_in(planet: &PlanetSpec) -> (Well, Ship, f64) {
    let mut well = Well::planet(planet, 0.0);
    well.lock = lock(planet);
    let out = DVec3::X;
    let rot = DQuat::from_rotation_arc(DVec3::NEG_Z, -out);
    let pos = well.pos + out * planet.radius * START;
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), pos, rot);
    ship.vel = well.vel;
    ship.warp = Warp::Active { speed: 0.0 };
    let (mut time, mut least) = (0.0, f64::INFINITY);
    while time < WARP_LIMIT {
        if well.pos.distance(ship.pos) <= radius(planet) {
            drop_out(&mut ship, &[well]);
            return (well, ship, least);
        }
        advance(&mut ship, &[well], CRUISE_CAP, DT);
        least = least.min(well.pos.distance(ship.pos));
        time += DT;
    }
    panic!("the warp to the planet never ended");
}

/// Returns the seconds a ship at rest `from` metres from the centre of `planet` takes to fly down to the
/// top of its air with the speed hold at its top speed, which the engine reaches within a third of it.
fn descent(planet: &PlanetSpec, from: f64) -> f64 {
    let (mut at, mut speed, mut time) = (from, 0.0, 0.0);
    let air = planet.radius + planet.atmosphere_height;
    while at > air && time < 3600.0 {
        let clearance = at - planet.radius;
        let orbital = (planet.mu() / at).sqrt();
        let top = top_speed_at(clearance, orbital);
        speed = top.min(speed + (top / 3.0).max(40.0) * DT);
        at -= speed * DT;
        time += DT;
    }
    time
}

/// Earth, a moon and a giant: the drop-out lies between the lock of the shell and the shell, at rest
/// beside the body, and the warp never came nearer than the lock.
#[test]
fn a_course_ends_at_the_shell_at_rest() {
    for planet in [earth(), moon(), giant()] {
        let (well, ship, least) = warp_in(&planet);
        let gap = well.pos.distance(ship.pos);
        assert!(gap >= lock(&planet) && gap <= radius(&planet), "{gap} m");
        assert_eq!(ship.warp, Warp::Off);
        assert!((ship.vel - well.vel).length() < 1e-9);
        assert!(least >= lock(&planet) * 0.999, "{least} m under warp");
    }
}

/// The lock of the shell is outside the air and off the ground, and the shell is under a third of a
/// radius above the air, so the flight in is short.
#[test]
fn the_shell_clears_the_air_and_hugs_the_body() {
    for planet in [earth(), moon(), giant()] {
        let air = planet.radius + planet.atmosphere_height;
        assert!(lock(&planet) > air && lock(&planet) > planet.radius * 1.01);
        assert!(radius(&planet) < planet.radius * 1.3 + planet.atmosphere_height * 1.2);
    }
}

/// From the drop-out the body is reached, to the top of its air, in under two minutes.
#[test]
fn the_air_is_reached_within_two_minutes() {
    for planet in [earth(), moon(), giant()] {
        let (well, ship, _) = warp_in(&planet);
        let seconds = descent(&planet, well.pos.distance(ship.pos));
        println!("{} m radius: {seconds:.0} s to the air", planet.radius);
        assert!(seconds < DESCENT_LIMIT, "{seconds} s");
    }
}
