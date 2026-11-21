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

/// What a ship did from its first touch: when it touched, when it rested, and the highest it rose after.
struct Set {
    /// Seconds into the flight of the first touch.
    touched: f64,
    /// Seconds into the flight of the rest.
    rested: f64,
    /// Highest height of the hull centre over the ground after the touch, in metres.
    highest: f64,
}

/// Sinks a ship with the stick held to the ground of `moon`, lets go at the first touch and flies 30 seconds.
fn sets_down(moon: &PlanetSpec) -> Option<Set> {
    let ground = |_: DVec3| 0.0;
    let scene = Scene {
        planet: Some(moon),
        ground: &ground,
        wells: &[],
        elapsed: 0.0,
    };
    let mut ship = assisted(moon, GEAR_DROP + 3.0);
    let (mut touched, mut highest, mut rested) = (None, 0.0f64, None);
    for index in 0..(30.0 / STEP) as usize {
        let time = index as f64 * STEP;
        let controls = touched.map_or_else(|| stick(-1.0), |_| Controls::default());
        let report = step(&mut ship, &controls, &scene, STEP);
        touched = touched.or((report.impact > 0.0).then_some(time));
        highest =
            highest.max(ship.pos.length() - moon.radius) * f64::from(u8::from(touched.is_some()));
        rested = rested.or(ship.landed.then_some(time));
    }
    Some(Set {
        touched: touched?,
        rested: rested?,
        highest,
    })
}

/// A ship that sinks with the stick held and lets go at the first touch, in the gravity of a moon, rests on
/// the ground within [`SETTLE`] seconds of the touch and never climbs more than [`HOP`] above where it
/// touched.
#[test]
fn a_released_ship_settles_on_a_moon() {
    let set = sets_down(&moon()).expect("a touch and a rest");
    let took = set.rested - set.touched;
    assert!(took < SETTLE, "rested {took} s after the touch");
    let hopped = set.highest >= GEAR_DROP + HOP;
    assert!(
        !hopped,
        "the ship hopped to {} m over the ground",
        set.highest
    );
}
