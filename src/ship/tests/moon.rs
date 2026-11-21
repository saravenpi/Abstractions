//! Setting down on a moon: the weak gravity makes the rebound of the gear long, and a ship whose stick is
//! released as it touches must settle on the ground, not take hold of the air and hop up again.

use bevy::math::DVec3;

use super::hover::{assisted, stick};
use super::planet;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::landing::GEAR_DROP;
use crate::universe::PlanetSpec;

/// Seconds a ship has to come to rest after it touches.
const SETTLE: f64 = 8.0;

/// Metres the ship may rise above the height of its first touch: a rebound, not a hop.
const HOP: f64 = 1.5;

/// A moon of 1.7 km radius with no air and the gravity of the Moon.
fn moon() -> PlanetSpec {
    PlanetSpec {
        gravity: 1.6,
        atmosphere_height: 0.0,
        ..planet(1.7e6, 0.0, 2.4e6)
    }
}

/// A ship that sinks with the stick held and lets go at the first touch, in the gravity of a moon, rests on
/// the ground within [`SETTLE`] seconds of the touch and never climbs more than [`HOP`] above where it
/// touched.
#[test]
fn a_released_ship_settles_on_a_moon() {
    let moon = moon();
    let ground = |_: DVec3| 0.0;
    let scene = Scene {
        planet: Some(&moon),
        ground: &ground,
        wells: &[],
        elapsed: 0.0,
    };
    let mut ship = assisted(&moon, GEAR_DROP + 3.0);
    let (mut touched, mut highest, mut rested) = (None::<f64>, 0.0f64, None::<f64>);
    for step_index in 0..(30.0 / STEP) as usize {
        let time = step_index as f64 * STEP;
        let controls = match touched {
            None => stick(-1.0),
            Some(_) => Controls::default(),
        };
        let report = step(&mut ship, &controls, &scene, STEP);
        if report.impact > 0.0 && touched.is_none() {
            touched = Some(time);
        }
        if touched.is_some() {
            highest = highest.max(ship.pos.length() - moon.radius);
        }
        if ship.landed && rested.is_none() {
            rested = Some(time);
        }
    }
    let (touch, rest) = (touched.expect("a touch"), rested.expect("a rest"));
    assert!(
        rest - touch < SETTLE,
        "rested {} s after the touch",
        rest - touch
    );
    let hopped = highest >= GEAR_DROP + HOP;
    assert!(!hopped, "the ship hopped to {highest} m over the ground");
    assert!(ship.landed, "the ship is not landed");
}
