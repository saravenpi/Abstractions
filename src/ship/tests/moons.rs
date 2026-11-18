//! Moons as bodies: their wells, the frame hand-over at the top of their air and the body the
//! speed hold flies beside, on a hand-built gas giant with one moon.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::two_steps;
use crate::ship::body::Ship;
use crate::ship::flight::STEP;
use crate::ship::frames::{planet_velocity, spin_rate};
use crate::ship::transit::reframe;
use crate::ship::warp::Warp;
use crate::ship::wells::{PLANET_LOCK, beside, nearest, system_wells};
use crate::space::Frame;
use crate::universe::{BodyId, PlanetClass, PlanetRecord, PlanetSpec, SystemSpec, Universe};
use crate::world::Link;

/// Radius of the hand-built giant in metres.
pub(super) const GIANT: f64 = 60_000.0;

/// Returns `planet` turned into a gas giant ten times its size with thick air and one moon.
pub(super) fn giant_of(planet: &PlanetSpec) -> PlanetSpec {
    PlanetSpec {
        radius: GIANT,
        gravity: 20.0,
        atmosphere_height: GIANT * 0.8,
        record: PlanetRecord {
            class: PlanetClass::GasGiant,
            pressure: 50.0,
            ocean: 0.0,
            ..planet.record
        },
        moons: 1,
        ..planet.clone()
    }
}

/// Returns the first moon of `giant`, `radius` metres from its centre, with a little air.
fn moon_of(giant: &PlanetSpec, radius: f64) -> PlanetSpec {
    let seconds = TAU * (radius.powi(3) / giant.mu()).sqrt();
    PlanetSpec {
        id: BodyId {
            moon: 1,
            ..giant.id
        },
        seed: giant.seed ^ 1,
        radius: 3000.0,
        gravity: 1.5,
        day_seconds: seconds,
        orbit_radius: radius,
        orbit_seconds: seconds,
        orbit_phase: 0.4,
        atmosphere_height: 1500.0,
        record: PlanetRecord::TEMPERATE,
        host: Some(giant.orbit()),
        moons: 0,
        rings: None,
        ..giant.clone()
    }
}

/// Returns the home system with its home planet made a gas giant, the rank of the giant, and
/// its moon `orbit` giant radii out, listed after the planets as the ship keeps a system.
pub(super) fn giant_system(orbit: f64) -> (SystemSpec, usize) {
    let universe = Universe::new(7);
    let mut system = universe.system(universe.home().star);
    let index = usize::from(universe.home().index);
    let giant = giant_of(&system.planets[index]);
    let moon = moon_of(&giant, orbit * GIANT);
    system.planets[index] = giant;
    system.planets.push(moon);
    (system, index)
}

/// The well of a moon is where its planet is plus its own orbit, moves with the velocity of
/// its planet plus its own, and has a lock sphere like a planet.
#[test]
fn a_moon_well_moves_with_its_planet() {
    let (system, index) = giant_system(2.5);
    let moon = system.planets.last().unwrap();
    for time in [0.0, 1234.5, 98_765.25] {
        let wells = system_wells(&system, time);
        let (giant, own) = (wells[index + 1], *wells.last().unwrap());
        let orbit = moon.orbit();
        let turn = orbit.at(time + 0.5) - orbit.at(time - 0.5);
        assert!((own.pos - giant.pos - orbit.at(time)).length() < 1e-6);
        assert!((own.vel - giant.vel - turn).length() < 1e-6, "{time}");
        assert!(turn.length() > 100.0 && own.lock == moon.radius * PLANET_LOCK);
    }
}

/// Returns the position and velocity of `ship` in the frame of the star of `moon` at `time`.
fn in_system(ship: &Ship, moon: &PlanetSpec, time: f64) -> (DVec3, DVec3) {
    if let Frame::System(_) = ship.frame {
        return (ship.pos, ship.vel);
    }
    let link = Link::orbiting(moon, time);
    let ground = (DVec3::Y * spin_rate(moon)).cross(ship.pos);
    let vel = planet_velocity(moon, time) + link.turn * (ship.vel + ground);
    (link.to_anchor(ship.pos), vel)
}

/// A ship that sinks into the air of a moon is handed to the surface frame of the moon, and
/// back to the frame of the star above it, without a jump in place or speed.
#[test]
fn the_frame_hands_over_into_and_out_of_a_moon_air() {
    let (system, _) = giant_system(2.5);
    let moon = system.planets.last().unwrap();
    let (time, out) = (4321.5, DVec3::new(0.48, 0.6, -0.64));
    let top = moon.radius + moon.atmosphere_height;
    let centre = moon.position(time);
    let mut ship = Ship::at_rest(
        Frame::System(moon.id.star),
        centre + out * (top - 10.0),
        DQuat::IDENTITY,
    );
    ship.vel = planet_velocity(moon, time) + DVec3::new(30.0, -12.0, 55.0);
    ship.warp = Warp::Spooling { left: 1.0 };
    let before = in_system(&ship, moon, time);
    reframe(&mut ship, &system, time);
    assert_eq!(
        (ship.frame, ship.warp),
        (Frame::Surface(moon.id), Warp::Off)
    );
    let after = in_system(&ship, moon, time);
    assert!((after.0 - before.0).length() < 1e-3 && (after.1 - before.1).length() < 1e-3);
    ship.pos = ship.pos.normalize() * (top + 80.0);
    let up = in_system(&ship, moon, time);
    reframe(&mut ship, &system, time);
    assert_eq!(ship.frame, Frame::System(moon.id.star));
    let back = in_system(&ship, moon, time);
    assert!((back.0 - up.0).length() < 1e-3 && (back.1 - up.1).length() < 1e-3);
}

/// Beside a moon the speed hold flies against the moon, though the ship is deep in the lock
/// sphere of its giant and that lock is the nearest; beside the giant it flies against the giant.
#[test]
fn the_speed_hold_picks_the_body_the_ship_is_beside() {
    let (system, index) = giant_system(2.5);
    let wells = system_wells(&system, 900.0);
    let (giant, moon) = (wells[index + 1], *wells.last().unwrap());
    let out = (moon.pos - giant.pos).normalize();
    let by_moon = moon.pos + out * moon.radius * 2.0;
    assert_eq!(beside(&wells, by_moon), Some(&moon));
    assert_eq!(nearest(&wells, by_moon), Some(&giant));
    let by_giant = giant.pos - out * (giant.radius + 5000.0);
    assert_eq!(beside(&wells, by_giant), Some(&giant));
}

/// With the assist on and the engines idle, a ship that drifts beside a moon comes to rest
/// relative to the moon, which runs around its giant at hundreds of metres per second.
#[test]
fn the_assist_stops_a_ship_beside_a_moon() {
    let (system, _) = giant_system(2.5);
    let moon = system.planets.last().unwrap();
    let mut time = 900.0;
    let at = moon.position(time) + moon.orbit().at(time).normalize() * moon.radius * 2.0;
    let mut ship = Ship::at_rest(Frame::System(moon.id.star), at, DQuat::IDENTITY);
    ship.vel = planet_velocity(moon, time) + DVec3::new(40.0, 0.0, -25.0);
    for _ in 0..(20.0 / (2.0 * STEP)) as usize {
        two_steps(&mut ship, &system_wells(&system, time));
        time += 2.0 * STEP;
    }
    let drift = (ship.vel - planet_velocity(moon, time)).length();
    assert!(drift < 2.0, "drifts at {drift} m/s past the moon");
}
