//! Setting down on a planet of any gravity: a ship that sinks with the stick held and lets go at the first
//! touch does not rise again, and rests within a few seconds.

use super::moon::sets_down;
use super::planet;
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
