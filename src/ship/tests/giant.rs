//! Flight over a gas giant: no ground, a floor the assist keeps above the cloud deck, and deep
//! air that floats a ship back up when the assist is off.

use std::f64::consts::TAU;

use bevy::math::{DQuat, DVec3};

use super::moons::giant_of;
use super::{above_pole, fly, planet};
use crate::ship::body::Ship;
use crate::ship::deck::{FLOOR, grounded};
use crate::ship::drag::density;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::landing::GEAR_DROP;
use crate::universe::{HOME_GRAVITY, PlanetSpec};

/// Returns the hand-built gas giant.
fn giant() -> PlanetSpec {
    giant_of(&planet(6000.0, 4000.0, 1200.0))
}

/// Returns the height in metres of the gear of a ship at `pos` above the deck of `planet`.
fn gear(planet: &PlanetSpec, pos: DVec3) -> f64 {
    pos.length() - planet.radius - GEAR_DROP
}

/// Under the deck of a giant the air keeps thickening, to a finite density however deep, while
/// under the sea of a solid planet it stays the air at sea level.
#[test]
fn the_air_of_a_giant_thickens_under_its_deck() {
    let (giant, home) = (giant(), planet(6000.0, 4000.0, 1200.0));
    let scale = giant.scale_height();
    let deck = density(&giant, 0.0);
    assert!(density(&giant, -scale) / deck > 2.7 && density(&giant, -scale) / deck < 2.8);
    assert!(density(&giant, -2.0 * scale) > density(&giant, -scale));
    assert!(density(&giant, -1e12).is_finite());
    assert_eq!(density(&home, -100.0), density(&home, 0.0));
}

/// With the assist on, a ship that dives at the deck of a giant with the stick down stops on
/// the floor above it and stays there, and it never lands.
#[test]
fn the_assist_holds_the_floor_over_a_giant() {
    let giant = giant();
    let mut ship = above_pole(&giant, 3000.0);
    (ship.assist, ship.throttle) = (true, 1.0);
    ship.rot = DQuat::from_rotation_x(-TAU / 6.0);
    let controls = Controls {
        lift: -1.0,
        boost: true,
        ..Controls::default()
    };
    let (mut lowest, mut landed) = (f64::INFINITY, false);
    fly(&mut ship, &controls, (&giant, &|_| 0.0), 90.0, |ship, _| {
        lowest = lowest.min(gear(&giant, ship.pos));
        landed |= ship.landed;
    });
    assert!(lowest > FLOOR - 0.05, "the gear sank to {lowest} m");
    assert!(
        gear(&giant, ship.pos) < FLOOR + 1.0,
        "held at {} m",
        gear(&giant, ship.pos)
    );
    assert!(!landed && ship.vel.length().is_finite());
}

/// A ship that comes landed onto a giant is in the air at the first step, and an idle ship at
/// the floor never comes to rest on it. Neither boarding nor stepping out is offered there.
#[test]
fn nothing_lands_on_a_giant() {
    let giant = giant();
    let mut ship = above_pole(&giant, FLOOR + GEAR_DROP);
    (ship.landed, ship.touching, ship.assist) = (true, true, true);
    let scene = Scene {
        planet: Some(&giant),
        ground: &|_| 0.0,
        wells: &[],
        elapsed: 0.0,
    };
    step(&mut ship, &Controls::default(), &scene, STEP);
    assert!(!ship.landed);
    let mut landed = false;
    fly(
        &mut ship,
        &Controls::default(),
        (&giant, &|_| 0.0),
        20.0,
        |ship, _| {
            landed |= ship.landed;
        },
    );
    assert!(!landed && (gear(&giant, ship.pos) - FLOOR).abs() < 0.05);
    assert!(!grounded(Some(&giant)) && grounded(Some(&planet(6000.0, 4000.0, 1200.0))));
    assert!(grounded(None));
}

/// Drives a ship without assist nose first into the deck of `giant` at full boost for two
/// minutes, and returns it with the lowest altitude it reached and the highest dynamic pressure.
fn plunge(giant: &PlanetSpec) -> (Ship, f64, f64) {
    let mut ship = above_pole(giant, 0.0);
    (ship.rot, ship.throttle) = (DQuat::from_rotation_x(-TAU / 4.0), 1.0);
    let boost = Controls {
        boost: true,
        ..Controls::default()
    };
    let (mut deepest, mut loudest) = (0.0f64, 0.0f64);
    fly(
        &mut ship,
        &boost,
        (giant, &|_| 0.0),
        120.0,
        |ship, report| {
            deepest = deepest.min(ship.pos.length() - giant.radius);
            loudest = loudest.max(report.pressure);
        },
    );
    (ship, deepest, loudest)
}

/// With the assist off, a ship driven nose first into the deck at full boost is stopped by the
/// thickening air far above the centre, buffeted hard on the way, and floats back up once the
/// engines idle.
#[test]
fn the_deep_air_floats_a_ship_back_up() {
    let giant = giant();
    let scale = giant.scale_height();
    let (mut ship, deepest, loudest) = plunge(&giant);
    assert!(
        deepest > -6.0 * scale && deepest < -scale,
        "sank to {deepest} m"
    );
    assert!(loudest > 2.0e4, "the hull felt only {loudest} Pa");
    ship.throttle = 0.0;
    fly(
        &mut ship,
        &Controls::default(),
        (&giant, &|_| 0.0),
        120.0,
        |_, _| {},
    );
    let risen = ship.pos.length() - giant.radius;
    assert!(
        risen > deepest + 1000.0 && risen < 0.0,
        "{deepest} m, then {risen} m"
    );
}

/// With the assist on and the controls let go, a ship started three scale heights under the
/// deck of a giant is floated up by the air to the floor within seven minutes and held there.
/// Past the first second the g readout shows the weight the ship feels, never the push of the
/// air against the engines that held it still before.
#[test]
fn the_deep_air_floats_an_assisted_ship_to_the_floor() {
    let giant = giant();
    let scale = giant.scale_height();
    let mut ship = above_pole(&giant, -3.0 * scale);
    (ship.assist, ship.throttle) = (true, 0.0);
    let (mut time, mut risen, mut most) = (0.0, None, 0.0f64);
    let land: super::super::atmo::Land = (&giant, &|_| 0.0);
    fly(
        &mut ship,
        &Controls::default(),
        land,
        480.0,
        |ship, report| {
            time += STEP;
            let felt = report.proper.length() / HOME_GRAVITY;
            most = most.max(if time > 1.0 { felt } else { 0.0 });
            if risen.is_none() && gear(&giant, ship.pos) > FLOOR - 1.0 {
                risen = Some(time);
            }
        },
    );
    let held = gear(&giant, ship.pos);
    println!("floated to the floor after {risen:?} s, then at most {most} g, held at {held} m");
    assert!(risen.is_some_and(|time| time < 420.0), "still at {held} m");
    assert!((held - FLOOR).abs() < 1.0, "held at {held} m");
    let weight = giant.gravity * (giant.radius / (giant.radius - 3.0 * scale)).powi(2);
    assert!(
        most < 1.2 * weight / HOME_GRAVITY,
        "the readout showed {most} g"
    );
}
