//! Handling near the ground with the assist on: a firm hover, steady rise and sink, levelling
//! on release, turning on the spot when landed, and no turn that puts the hull into the ground.

use bevy::math::{DQuat, DVec3};

use super::hover::{assisted, stick};
use super::{coast, fly, planet};
use crate::ship::body::Ship;
use crate::ship::cruise::HOVER_RATES;
use crate::ship::flight::Controls;
use crate::ship::landing::{GEAR_DROP, HULL};
use crate::universe::PlanetSpec;

/// Returns the height of a slope of 20 degrees that rises toward +X around the +Y pole.
fn hillside(p: DVec3) -> f64 {
    200.0 + 6000.0 * p.x.atan2(p.y) * 20.0f64.to_radians().tan()
}

/// Returns controls with the stick at `turn` and everything else released.
fn turning(turn: DVec3) -> Controls {
    Controls {
        turn,
        ..Controls::default()
    }
}

/// Returns the degrees between the roof of `ship` and the local up.
fn tilt(ship: &Ship) -> f64 {
    (ship.rot * DVec3::Y)
        .dot(ship.pos.normalize())
        .min(1.0)
        .acos()
        .to_degrees()
}

/// Returns how far in metres the lowest hull point of `ship` is above the ground `height`.
fn hull_clearance(ship: &Ship, planet: &PlanetSpec, height: &dyn Fn(DVec3) -> f64) -> f64 {
    let above = |body: DVec3| {
        let at = ship.pos + ship.rot * body;
        at.length() - planet.radius - height(at)
    };
    HULL.into_iter().map(above).fold(f64::INFINITY, f64::min)
}

/// The rise key climbs at a steady 10 m/s and the sink key sinks at a steady 6 m/s, and once
/// the rise key is let go the ship holds its height within 5 cm, still.
#[test]
fn rise_and_sink_are_steady_and_the_hover_is_firm() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 30.0);
    coast(&mut ship, &stick(1.0), land, 2.0);
    let up = ship.pos.normalize();
    assert!(
        (ship.vel.dot(up) - HOVER_RATES.0).abs() < 0.05,
        "{}",
        ship.vel.dot(up)
    );
    coast(&mut ship, &Controls::default(), land, 2.0);
    let held = ship.pos.length();
    let (mut bob, mut speed) = (0.0f64, 0.0f64);
    fly(&mut ship, &Controls::default(), land, 10.0, |ship, _| {
        bob = bob.max((ship.pos.length() - held).abs());
        speed = speed.max(ship.vel.length());
    });
    assert!(bob < 0.05 && speed < 0.05, "bob {bob} m at {speed} m/s");
    coast(&mut ship, &stick(-1.0), land, 1.5);
    assert!(
        (ship.vel.dot(up) + HOVER_RATES.1).abs() < 0.05,
        "{}",
        ship.vel.dot(up)
    );
}

/// A ship under way that is near the ground, slow and tilted by 20 degrees levels within two
/// seconds once the stick is let go. High above the ground the same tilt is kept.
#[test]
fn a_released_stick_levels_near_the_ground() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    for (height, kept) in [(60.0, false), (2500.0, true)] {
        let mut ship = assisted(&planet, height);
        ship.rot = DQuat::from_rotation_x(20.0f64.to_radians());
        ship.throttle = 0.3;
        coast(&mut ship, &Controls::default(), land, 2.0);
        let leaning = tilt(&ship);
        if kept {
            assert!(leaning > 15.0, "{height} m: levelled to {leaning}");
        } else {
            assert!(leaning < 0.5, "{height} m: still leaning by {leaning}");
        }
    }
}

/// Landed on a 20 degree slope, the yaw and the roll keys turn the ship on the spot without
/// any hull point going under the ground, the pitch key does nothing and the ship stays landed.
#[test]
fn a_landed_ship_only_turns_on_the_spot() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &hillside as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 200.0 + GEAR_DROP + 6.0);
    coast(&mut ship, &stick(-1.0), land, 20.0);
    assert!(ship.landed);
    let (rested, nose) = (ship.clone(), ship.forward());
    coast(&mut ship, &turning(DVec3::X), land, 2.0);
    assert_eq!(
        (ship.pos, ship.rot, ship.landed),
        (rested.pos, rested.rot, true)
    );
    for turn in [DVec3::Y, DVec3::NEG_Z] {
        let mut lowest = f64::INFINITY;
        fly(&mut ship, &turning(turn), land, 3.0, |ship, _| {
            lowest = lowest.min(hull_clearance(ship, &planet, &hillside));
        });
        assert!(
            ship.landed && ship.pos == rested.pos && lowest > 0.0,
            "{turn}: {lowest}"
        );
    }
    assert!(
        ship.forward().dot(nose) > 0.99,
        "yaw left and roll right did not turn back"
    );
    coast(&mut ship, &turning(DVec3::Y), land, 2.0);
    assert!(ship.forward().angle_between(nose) > 0.8, "did not turn");
}

/// Hovering low beside a 20 degree slope, a full pitch or roll held for three seconds either
/// way never puts a hull point under the ground.
#[test]
fn no_turn_puts_the_hull_into_a_slope() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &hillside as &dyn Fn(DVec3) -> f64);
    for turn in [DVec3::X, DVec3::NEG_X, DVec3::Z, DVec3::NEG_Z] {
        let mut ship = assisted(&planet, 200.0 + GEAR_DROP + 1.5);
        coast(&mut ship, &Controls::default(), land, 3.0);
        let mut lowest = f64::INFINITY;
        fly(&mut ship, &turning(turn), land, 3.0, |ship, _| {
            lowest = lowest.min(hull_clearance(ship, &planet, &hillside));
        });
        assert!(
            lowest > 0.0,
            "{turn}: a hull point went {lowest} m into the ground"
        );
    }
}
