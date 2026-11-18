//! Ground tests: resting on a slope, hard and soft landings, floating and the saved record.

use bevy::math::DVec3;

use super::{above_pole, coast, fly, planet};
use crate::ship::berth::{landing_spot, parked};
use crate::ship::body::{Ship, ShipState};
use crate::ship::flight::Controls;
use crate::ship::landing::GEAR_DROP;
use crate::universe::PlanetSpec;

/// Returns the height of a slope of `degrees` that rises toward +X around the +Y pole of `planet`.
fn slope(planet: &PlanetSpec, degrees: f64) -> impl Fn(DVec3) -> f64 {
    let (radius, rise) = (planet.radius, degrees.to_radians().tan());
    move |p: DVec3| 200.0 + radius * p.x.atan2(p.y) * rise
}

/// A ship set down on a 10 degree slope comes to rest and stays put for 60 seconds.
#[test]
fn landed_ship_holds_on_a_slope() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let ground = slope(&planet, 10.0);
    let mut ship = above_pole(&planet, 200.0 + GEAR_DROP + 0.3);
    let start = ship.pos;
    coast(&mut ship, &Controls::default(), (&planet, &ground), 6.0);
    assert!(ship.landed, "still moving at {}", ship.vel.length());
    assert!(
        (ship.pos - start).length() < 1.5,
        "slid {}",
        (ship.pos - start).length()
    );
    let (rested, tilt) = (
        ship.pos,
        (ship.rot * DVec3::Y).dot(DVec3::Y).acos().to_degrees(),
    );
    assert!((tilt - 10.0).abs() < 3.0, "tilted by {tilt}");
    coast(&mut ship, &Controls::default(), (&planet, &ground), 60.0);
    assert!(ship.landed && (ship.pos - rested).length() < 1e-3);
    assert_eq!(ship.damage, 0.0);
}

/// A slope steeper than the friction limit does not hold the ship.
#[test]
fn steep_slope_lets_the_ship_slide() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let ground = slope(&planet, 40.0);
    let mut ship = above_pole(&planet, 200.0 + GEAR_DROP + 0.3);
    let start = ship.pos;
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &ground),
        8.0,
        |_, _| {},
    );
    assert!(!ship.landed && (ship.pos - start).length() > 5.0);
}

/// A drop from 40 metres stops dead on the ground instead of bouncing, reports damage, and the
/// ship is still in one piece on the ground. Until 0.9 it bounced back up at over 2 m/s.
#[test]
fn hard_landing_stops_dead_and_damages() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 40.0);
    let (mut hit, mut rebound) = (0.0f64, 0.0f64);
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| 0.0),
        30.0,
        |ship, report| {
            hit = hit.max(report.impact);
            rebound = rebound.max(ship.vel.y);
        },
    );
    assert!(hit > 20.0, "touched down at {hit}");
    assert!(rebound < 1.0, "rebound of {rebound}");
    assert!(
        ship.damage > 0.2 && ship.damage <= 1.0,
        "damage {}",
        ship.damage
    );
    assert!(ship.landed && (ship.pos.length() - planet.radius - GEAR_DROP).abs() < 0.5);
}

/// With the assist on, holding the thrusters down from a hover lands under the safe speed.
#[test]
fn assist_hovers_and_lands_softly() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 60.0);
    ship.assist = true;
    coast(&mut ship, &Controls::default(), (&planet, &|_| 0.0), 10.0);
    let held = ship.pos.length() - planet.radius;
    assert!(
        (held - 60.0).abs() < 1.0 && ship.vel.length() < 0.1,
        "hover at {held}"
    );
    let down = Controls {
        lift: -1.0,
        ..Controls::default()
    };
    let mut hit = 0.0f64;
    fly(&mut ship, &down, (&planet, &|_| 0.0), 40.0, |_, report| {
        hit = hit.max(report.impact)
    });
    assert!(
        ship.landed && ship.damage == 0.0,
        "landed {} hit {hit}",
        ship.landed
    );
    assert!(hit > 0.5 && hit < 6.0, "touched down at {hit}");
    let up = Controls {
        lift: 1.0,
        ..Controls::default()
    };
    fly(&mut ship, &up, (&planet, &|_| 0.0), 3.0, |_, _| {});
    assert!(!ship.landed && ship.pos.length() - planet.radius > 10.0);
}

/// Over deep water the ship floats at the surface and comes to rest.
#[test]
fn ship_floats_on_the_sea() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 4.0);
    fly(
        &mut ship,
        &Controls::default(),
        (&planet, &|_| -80.0),
        30.0,
        |_, _| {},
    );
    let height = ship.pos.length() - planet.radius;
    assert!(
        ship.landed && height > 0.0 && height < GEAR_DROP,
        "floating at {height}"
    );
    assert_eq!(ship.damage, 0.0);
}

/// The saved record survives a trip through toml and rebuilds the same ship.
#[test]
fn state_round_trips_through_toml() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, 12.5);
    ship.vel = DVec3::new(1.5, -2.0, 0.25);
    (ship.landed, ship.touching, ship.assist) = (true, true, true);
    let text = toml::to_string(&ship.state()).unwrap();
    let state: ShipState = toml::from_str(&text).unwrap();
    assert_eq!(state, ship.state());
    assert_eq!(Ship::from_state(&state), Some(ship));
}

/// A new ship stands on its gear on dry, level ground near the spawn point, not in the bay beside it.
#[test]
fn ship_starts_on_dry_flat_ground() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let near = DVec3::new(0.3, 0.9, -0.2).normalize() * planet.radius;
    let ground = |p: DVec3| {
        let from = (p.normalize() * planet.radius).distance(near);
        if from < 40.0 { -3.0 } else { 5.0 + 0.02 * from }
    };
    let spot = landing_spot(&ground, near, 0.0);
    let from = spot.distance(near);
    assert!(from > 44.0 && from < 120.0, "spot {from} m away");
    let ship = parked(planet.id, (&ground, 0.0), spot, planet.radius);
    let up = spot.normalize();
    assert!(ship.landed && (ship.rot * DVec3::Y).dot(up) > 0.999_999);
    assert!((ship.pos.length() - planet.radius - ground(spot) - GEAR_DROP).abs() < 1e-6);
    let wet = landing_spot(&|_| -3.0, near, 0.0);
    assert!(wet.distance(near) < 20.0 && (wet.length() - planet.radius).abs() < 1e-6);
}
