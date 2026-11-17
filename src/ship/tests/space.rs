//! Flight tests away from the ground: orbits, free fall, terminal velocity and frame changes.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::{above_pole, fly, planet, two_steps};
use crate::ship::body::Ship;
use crate::ship::drag::{DRAG, density};
use crate::ship::flight::{Controls, STEP};
use crate::ship::frames::{planet_velocity, spin_rate, to_surface, to_system};
use crate::ship::orbit::apsides;
use crate::ship::transit::{catch_up, reframe};
use crate::ship::warp::Warp;
use crate::ship::wells::system_wells;
use crate::space::Frame;
use crate::universe::Universe;

/// Returns the speed in metres per second of a circular orbit of radius `r` metres.
fn circular_speed(mu: f64, r: f64) -> f64 {
    (mu / r).sqrt()
}

/// A circular orbit at 1.2 radii in the turning surface frame keeps its radius over 10 orbits.
#[test]
fn surface_frame_orbit_keeps_its_radius() {
    let planet = planet(6000.0, 0.0, 1200.0);
    let r = planet.radius * 1.2;
    let speed = circular_speed(planet.mu(), r);
    let mut ship = above_pole(&planet, 0.0);
    ship.pos = DVec3::X * r;
    let inertial = DVec3::new(0.0, 0.6, 0.8) * speed;
    ship.vel = inertial - (DVec3::Y * spin_rate(&planet)).cross(ship.pos);
    let period = TAU * r / speed;
    let mut worst = 0.0f64;
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| 0.0),
        10.0 * period,
        |ship, _| {
            worst = worst.max((ship.pos.length() / r - 1.0).abs());
        },
    );
    assert!(worst < 0.01, "radius drifted by {worst}");
}

/// The same orbit in the frame of the star, around the moving planet, keeps its radius and its apsides.
#[test]
fn system_frame_orbit_keeps_its_radius() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let planet = &system.planets[usize::from(universe.home().index)];
    let r = planet.radius * 1.2;
    let speed = circular_speed(planet.mu(), r);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), DVec3::ZERO, DQuat::IDENTITY);
    ship.pos = planet.position(0.0) + DVec3::Y * r;
    ship.vel = planet_velocity(planet, 0.0) + DVec3::X * speed;
    let (mut worst, mut time) = (0.0f64, 0.0);
    for _ in 0..(10.0 * TAU * r / speed / (2.0 * STEP)) as usize {
        two_steps(&mut ship, &system_wells(&system, time));
        time += 2.0 * STEP;
        worst = worst.max(((ship.pos - planet.position(time)).length() / r - 1.0).abs());
    }
    assert!(worst < 0.01, "radius drifted by {worst}");
    let rel = (
        ship.pos - planet.position(time),
        ship.vel - planet_velocity(planet, time),
    );
    let (apo, peri) = apsides(rel.0, rel.1, planet.mu()).unwrap();
    assert!(
        (apo / r - 1.0).abs() < 0.01 && (peri / r - 1.0).abs() < 0.01,
        "{apo} {peri}"
    );
}

/// Without air a fall of one kilometre ends at `sqrt(2 g h)`.
#[test]
fn free_fall_matches_the_closed_form() {
    let planet = planet(40_000.0, 0.0, 1e9);
    let mut ship = above_pole(&planet, 1020.0);
    let mut speed = 0.0;
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| 0.0),
        30.0,
        |ship, _| {
            let fallen = planet.radius + 1020.0 - ship.pos.length();
            if fallen <= 1000.0 {
                speed = ship.vel.length();
            }
        },
    );
    let expected = (2.0 * planet.gravity * 1000.0f64).sqrt();
    assert!(
        (speed / expected - 1.0).abs() < 0.02,
        "{speed} against {expected}"
    );
}

/// With air a nose-down fall settles at the speed where drag equals weight.
#[test]
fn terminal_velocity_matches_the_closed_form() {
    let planet = planet(40_000.0, 80_000.0, 1e9);
    let mut ship = above_pole(&planet, 4000.0);
    ship.rot = DQuat::from_rotation_x(-TAU / 4.0);
    let mut seen = (0.0, 0.0);
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| 0.0),
        60.0,
        |ship, _| {
            let height = ship.pos.length() - planet.radius;
            if height >= 1000.0 {
                seen = (ship.vel.length(), height);
            }
        },
    );
    let pull = planet.mu() / (planet.radius + seen.1).powi(2);
    let expected = (pull / (DRAG.z * density(&planet, seen.1))).sqrt();
    assert!(seen.1 < 1010.0 && seen.0.is_finite(), "{seen:?}");
    assert!(
        (seen.0 / expected - 1.0).abs() < 0.05,
        "{} against {expected}",
        seen.0
    );
}

/// Surface to system and back gives the same ship, and the system velocity holds the ground speed.
#[test]
fn frame_round_trip_is_exact() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 3000.0);
    ship.pos = DVec3::new(5200.0, 3100.0, -6400.0);
    ship.vel = DVec3::new(120.0, -45.0, 310.0);
    ship.rot = DQuat::from_euler(bevy::math::EulerRot::YXZ, 0.7, -0.3, 1.9);
    ship.spin = DVec3::new(0.2, -0.1, 0.4);
    let (start, time) = (ship.clone(), 54_321.5);
    to_system(&mut ship, &planet, time);
    assert_eq!(ship.frame, Frame::System(planet.id.star));
    let ground = (DVec3::Y * spin_rate(&planet)).cross(start.pos) + start.vel;
    let moved = ship.vel - planet_velocity(&planet, time);
    assert!((moved.length() - ground.length()).abs() < 1e-6);
    to_surface(&mut ship, &planet, time);
    assert_eq!(ship.frame, start.frame);
    assert!(
        (ship.pos - start.pos).length() < 1e-3,
        "{}",
        ship.pos - start.pos
    );
    assert!(
        (ship.vel - start.vel).length() < 1e-3,
        "{}",
        ship.vel - start.vel
    );
    assert!((ship.spin - start.spin).length() < 1e-9);
    assert!(ship.rot.angle_between(start.rot) < 1e-9);
}

/// A climbing ship leaves the surface frame above the atmosphere and comes back to it below,
/// at the same place beside the planet. A landed ship never leaves it.
#[test]
fn ship_changes_frame_at_the_atmosphere_top() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let planet = &system.planets[usize::from(universe.home().index)];
    let (top, time) = (planet.radius + planet.atmosphere_height, 777.0);
    let mut ship = above_pole(planet, planet.atmosphere_height - 10.0);
    reframe(&mut ship, &system, time);
    assert_eq!(ship.frame, Frame::Surface(planet.id));
    ship.pos = DVec3::new(0.6, 0.0, 0.8) * (top + 80.0);
    ship.warp = Warp::Spooling { left: 1.0 };
    let mut parked = ship.clone();
    parked.landed = true;
    reframe(&mut parked, &system, time);
    assert_eq!(parked.frame, Frame::Surface(planet.id));
    reframe(&mut ship, &system, time);
    assert_eq!(ship.frame, Frame::System(planet.id.star));
    let height = ship.pos.distance(planet.position(time)) - top;
    assert!((height - 80.0).abs() < 1e-6 && ship.warp != Warp::Off);
    reframe(&mut ship, &system, time);
    assert_eq!(ship.frame, Frame::System(planet.id.star));
    ship.pos = planet.position(time) + (ship.pos - planet.position(time)) * 0.97;
    reframe(&mut ship, &system, time);
    assert_eq!(
        (ship.frame, ship.warp),
        (Frame::Surface(planet.id), Warp::Off)
    );
}

/// After a pause a ship near a planet is still beside it with the same relative velocity.
#[test]
fn paused_ship_stays_with_its_planet() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let planet = &system.planets[usize::from(universe.home().index)];
    let offset = DVec3::new(2.0, 1.0, -3.0) * planet.radius;
    let drift = DVec3::new(40.0, -5.0, 90.0);
    let mut ship = Ship::at_rest(
        Frame::System(planet.id.star),
        planet.position(100.0) + offset,
        DQuat::IDENTITY,
    );
    ship.vel = planet_velocity(planet, 100.0) + drift;
    catch_up(&mut ship, &system, 400.0, 300.0);
    assert!((ship.pos - planet.position(400.0) - offset).length() < 1e-3);
    assert!((ship.vel - planet_velocity(planet, 400.0) - drift).length() < 1e-6);
    let far = planet.position(400.0) * 3.0;
    let mut lone = Ship::at_rest(Frame::System(planet.id.star), far, DQuat::IDENTITY);
    catch_up(&mut lone, &system, 700.0, 300.0);
    assert_eq!(lone.pos, far);
}
