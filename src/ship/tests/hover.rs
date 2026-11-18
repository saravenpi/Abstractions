//! The flight assist over a planet: a released ship hovers still and level, flies level around
//! the globe and comes down gently.

use bevy::math::{DQuat, DVec3, EulerRot};

use super::{above_pole, coast, fly, planet};
use crate::ship::body::Ship;
use crate::ship::flight::Controls;
use crate::universe::PlanetSpec;

/// Returns a ship with the assist on `height` metres above the sea at the +Y pole of `planet`.
pub(super) fn assisted(planet: &PlanetSpec, height: f64) -> Ship {
    let mut ship = above_pole(planet, height);
    ship.assist = true;
    ship
}

/// Returns controls with the vertical stick at `lift` and everything else released.
pub(super) fn stick(lift: f64) -> Controls {
    Controls {
        lift,
        ..Controls::default()
    }
}

/// Returns the degrees between the roof of `ship` and the local up.
fn tilt(ship: &Ship) -> f64 {
    let roof = ship.rot * DVec3::Y;
    roof.dot(ship.pos.normalize()).min(1.0).acos().to_degrees()
}

/// Returns how far in metres a ship let go at `height` metres, moving and banked, still moves
/// up or down and sideways and how fast it still is, from 2 to 22 seconds after, and its tilt
/// in degrees at 2 seconds.
fn let_go(height: f64, assist: bool) -> (f64, f64, f64, f64) {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, height);
    (ship.assist, ship.vel) = (assist, DVec3::new(8.0, -3.0, 5.0));
    ship.rot = DQuat::from_euler(EulerRot::YXZ, 0.4, 0.26, 0.44);
    coast(&mut ship, &Controls::default(), land, 2.0);
    let (start, leaning) = (ship.pos, tilt(&ship));
    let (mut bob, mut slide, mut speed) = (0.0f64, 0.0f64, 0.0f64);
    fly(&mut ship, &Controls::default(), land, 20.0, |ship, _| {
        bob = bob.max((ship.pos.length() - start.length()).abs());
        slide = slide.max(ship.pos.distance(start));
        speed = speed.max(ship.vel.length());
    });
    (bob, slide, speed, leaning)
}

/// Two seconds after everything is released the ship hangs in the air: under 5 cm of motion
/// up, down or sideways and under 0.1 m/s over the next 20 seconds, and level within half a
/// degree. That holds near the ground and high above it. Without the assist it falls.
#[test]
fn a_released_ship_hovers_still_and_level() {
    for height in [60.0, 1500.0] {
        let (bob, slide, speed, leaning) = let_go(height, true);
        assert!(bob < 0.05 && slide < 0.05, "{height} m: {bob} {slide}");
        assert!(speed < 0.1, "{height} m: still moving at {speed} m/s");
        assert!(leaning < 0.5, "{height} m: leaning by {leaning} degrees");
    }
    let (fell, _, _, banked) = let_go(1500.0, false);
    assert!(
        fell > 500.0 && banked > 10.0,
        "fell {fell} m, banked {banked}"
    );
}

/// At half lever a level ship keeps its height within 2 m for a minute on a planet 6 km in
/// radius, and its roof stays up: the assist turns the ship and its path with the ground.
#[test]
fn level_flight_follows_the_curve_of_the_planet() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = assisted(&planet, 500.0);
    ship.throttle = 0.5;
    let (start, mut off) = (ship.pos, 0.0f64);
    let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    fly(&mut ship, &Controls::default(), land, 60.0, |ship, _| {
        off = off.max((ship.pos.length() - start.length()).abs());
    });
    let flown = ship.pos.distance(start);
    assert!(off < 2.0, "the height changed by {off} m");
    assert!(
        flown > 3000.0 && tilt(&ship) < 0.1,
        "{flown} m, {}",
        tilt(&ship)
    );
}

/// A ship that dives at the ground at full lever is slowed on the way: it meets the ground
/// under the 2 m/s that harm the hull, and takes no damage.
#[test]
fn a_full_lever_dive_meets_the_ground_gently() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    for dive in [0.5f64, 1.0, 1.5] {
        let mut ship = assisted(&planet, 2000.0);
        (ship.throttle, ship.rot) = (1.0, DQuat::from_rotation_x(-dive));
        let (mut first, mut fastest) = (None, 0.0f64);
        let land = (&planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
        fly(
            &mut ship,
            &Controls::default(),
            land,
            60.0,
            |ship, report| {
                first = first.or((report.impact > 0.0).then_some((report.impact, ship.damage)));
                fastest = fastest.max(ship.vel.length());
            },
        );
        let (hit, damage) = first.expect("the ship reaches the ground");
        assert!(hit < 2.0 && damage == 0.0, "dive {dive}: hit at {hit} m/s");
        assert!(fastest > 200.0, "dive {dive}: no faster than {fastest} m/s");
    }
}
