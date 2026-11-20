//! Tests of the climb of a route that goes elsewhere: the orbit it leaves from lies above the lock
//! sphere of the planet, where the warp drive is free.

use super::ascent::{altitude, departure};
use super::ascent_tests::{climb_from, earth};
use super::block::{Reading, status};
use super::handoff::reached;
use crate::computer::{RouteStatus, Target};
use crate::ship::pilot::{Autopilot, Phase};
use crate::ship::wells::Well;
use crate::universe::{PlanetSpec, Universe};

/// Seconds the climb to the orbit a route to another place leaves from may take.
const LEAVE_LIMIT: f64 = 900.0;

/// A route to another place leaves from an orbit above the lock sphere of the planet, so the
/// warp drive is free there: higher than the orbit of a route that ends at the planet, for a
/// planet whose lock sphere is wider than that orbit is high.
#[test]
fn the_departure_orbit_is_above_the_lock_sphere() {
    let planet = earth();
    let lock = Well::planet(&planet, 0.0).lock;
    assert!(planet.radius + altitude(&planet) < lock);
    assert!(planet.radius + departure(&planet) > lock);
    let tiny = PlanetSpec {
        radius: 1.0e5,
        ..planet
    };
    assert_eq!(departure(&tiny), altitude(&tiny));
}

/// The climb to the departure orbit ends in a circular orbit outside the lock sphere.
#[test]
fn the_climb_to_the_departure_orbit_leaves_the_drive_free() {
    let planet = earth();
    let (ship, time, arrived) = climb_from(&planet, (departure(&planet), LEAVE_LIMIT));
    assert!(arrived, "no orbit after {time} s");
    let lock = Well::planet(&planet, 0.0).lock;
    assert!(ship.pos.length() > lock, "{}", ship.pos.length());
}

/// A route to another place that reaches the departure orbit goes on to turn toward the place:
/// the route reads as aligning, with the target kept. A route to the planet itself has arrived.
#[test]
fn the_route_turns_toward_its_place_after_the_climb() {
    let universe = Universe::new(7);
    let (home, star) = (
        Target::Body(universe.home()),
        Target::Star(universe.home().star),
    );
    let mut pilot = Autopilot::default();
    pilot.engage(star);
    pilot.phase = Phase::Climbing;
    pilot.leaving = Some(universe.home());
    reached(&mut pilot, false);
    assert_eq!((pilot.target(), pilot.leaving), (Some(star), None));
    let reading = Reading {
        phase: pilot.phase(),
        block: None,
        aligned: 0,
        course: true,
        seated: true,
    };
    assert_eq!(status(reading), RouteStatus::Aligning(0));
    let mut stay = Autopilot::default();
    stay.engage(home);
    reached(&mut stay, true);
    assert_eq!((stay.target(), stay.phase()), (None, Phase::Arrived));
}
