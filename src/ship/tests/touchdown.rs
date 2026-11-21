//! Setting down on a planet of any gravity: a ship that sinks with the stick held and lets go at the first
//! touch does not rise again, and rests within a few seconds.

use bevy::math::DVec3;

use super::hover::assisted;
use super::moon::sets_down;
use super::planet;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::landing::GEAR_DROP;
use crate::universe::{PlanetSpec, STANDARD_GRAVITY};

/// Metres the hull centre may rise above the height of its first touch.
const RISE: f64 = 0.3;

/// Seconds the ship has to come to rest after the touch.
const SETTLE: f64 = 5.0;

/// Returns the standard planet of 6 km made `share` of the standard gravity.
fn weighing(share: f64) -> PlanetSpec {
    PlanetSpec {
        gravity: STANDARD_GRAVITY * share,
        ..planet(6000.0, 0.0, 1200.0)
    }
}

/// At half, one and one and a half times the standard gravity the ship settles: it never climbs
/// more than [`RISE`] over its first touch, and is landed within [`SETTLE`] seconds.
#[test]
fn a_released_ship_settles_at_every_gravity() {
    for share in [0.5, 1.0, 1.5] {
        let set = sets_down(&weighing(share)).expect("a touch and a rest");
        let rose = set.highest - GEAR_DROP;
        assert!(rose < RISE, "{share} g: the ship rose {rose} m");
        let took = set.rested - set.touched;
        assert!(took < SETTLE, "{share} g: rested {took} s after the touch");
    }
}

/// A ship that settles on its gear with the lever left at 0.4 and no rise asked is set down all the same: the
/// lever goes back to idle, the ship rests within [`SETTLE`] seconds and never rises more than [`RISE`].
#[test]
fn a_lever_left_up_does_not_keep_the_ship_from_resting() {
    let planet = weighing(1.0);
    let ground = |_: DVec3| 0.0;
    let scene = Scene {
        planet: Some(&planet),
        ground: &ground,
        wells: &[],
        elapsed: 0.0,
    };
    let mut ship = assisted(&planet, GEAR_DROP - 0.02);
    ship.throttle = 0.4;
    let (mut highest, mut rested) = (0.0f64, None);
    for index in 0..(10.0 / STEP) as usize {
        step(&mut ship, &Controls::default(), &scene, STEP);
        highest = highest.max(ship.pos.length() - planet.radius);
        rested = rested.or(ship.landed.then_some(index as f64 * STEP));
    }
    let took = rested.expect("a rest");
    assert!(took < SETTLE, "rested after {took} s");
    assert!(highest - GEAR_DROP < RISE, "the ship rose to {highest} m");
    assert_eq!(ship.throttle, 0.0);
}
