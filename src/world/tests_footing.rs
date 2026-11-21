//! Tests of the height a saved place may have: what stays and what is put on the ground.

use bevy::math::DVec3;

use super::Globe;
use super::footing::{aboard, grounded};
use crate::saves::Place;
use crate::universe::Universe;

/// Height in metres above sea level where the air of the tests ends.
const AIR: f64 = 1.0e5;

/// Returns the home planet of seed 4, a place at its sea level in a direction where the ground
/// is above the sea, and the height of the ground there.
fn home() -> (Globe, Place, f64) {
    let universe = Universe::new(4);
    let body = universe.home();
    let globe = Globe::of(&universe.planet(body));
    let sea = f64::from(globe.sea_level());
    let mut dirs = (0..400).map(|i| DVec3::new(0.3, 1.0, f64::from(i) * 0.01 - 2.0).normalize());
    let dry = |dir: &DVec3| f64::from(globe.height(*dir)) > sea + 5.0;
    let dir = dirs.find(dry).expect("dry land");
    let place = Place {
        body,
        dir: dir.to_array(),
        altitude: 0.0,
    };
    let ground = f64::from(globe.height(dir));
    (globe, place, ground)
}

/// A place on the ground, just under it and in the air stays as it is: the air is a place a
/// body can be at, such as the 6560 m the owner was saved at.
#[test]
fn a_place_on_the_ground_or_in_the_air_stays() {
    let (globe, place, ground) = home();
    for lift in [-2.0, 0.9, 174.0, 6560.18] {
        let at = Place {
            altitude: ground + lift,
            ..place
        };
        assert_eq!(grounded(at, &globe, AIR), at, "{lift}");
    }
}

/// A place deep under the ground, such as the lowest height a server repairs to, a height that
/// is not a number and a height beyond where the planet is drawn go to the ground in the same
/// direction.
#[test]
fn a_place_no_body_can_be_at_goes_to_the_ground() {
    let (globe, place, ground) = home();
    let beyond = globe.handover() - globe.radius() + 1.0;
    for altitude in [
        -20_000.0,
        ground - 40.0,
        f64::NAN,
        f64::INFINITY,
        beyond,
        AIR + 10.0,
    ] {
        let fixed = grounded(Place { altitude, ..place }, &globe, AIR);
        assert_eq!((fixed.body, fixed.dir), (place.body, place.dir));
        let above = fixed.altitude - ground;
        assert!((1.0..2.0).contains(&above), "{altitude}: {above}");
        assert_eq!(grounded(fixed, &globe, AIR), fixed);
    }
}

/// A place that points nowhere is never moved: the world drops it before it starts.
#[test]
fn a_place_that_points_nowhere_is_left_alone() {
    let (globe, place, _) = home();
    let lost = Place {
        dir: [0.0; 3],
        altitude: f64::NAN,
        ..place
    };
    assert!(grounded(lost, &globe, AIR).altitude.is_nan());
    assert!(aboard(lost, &globe).altitude.is_nan());
}

/// Without air to fly in, a body on foot is left within a hop of the ground and put on the
/// ground from higher: it is never left falling where the wingsuit carries nothing.
#[test]
fn a_body_on_foot_without_air_is_put_on_the_ground() {
    let (globe, place, ground) = home();
    let at = |lift: f64| Place {
        altitude: ground + lift,
        ..place
    };
    assert_eq!(grounded(at(30.0), &globe, 0.0), at(30.0));
    assert_eq!(grounded(at(500.0), &globe, 0.0), at(1.5));
    assert_eq!(grounded(at(500.0), &globe, AIR), at(500.0));
}

/// The place of a player seated in the ship is kept at any height, in orbit included, and only
/// brought within what the world loads: no lower than the ground, no higher than just past
/// where the planet stops being drawn, which is where open space and a warp load.
#[test]
fn a_seated_place_is_kept_at_any_height() {
    let (globe, place, ground) = home();
    let at = |altitude: f64| Place { altitude, ..place };
    let top = (globe.handover() - globe.radius()) * 1.01;
    for altitude in [ground + 2.0, ground + 6560.0, globe.radius() * 0.2, top] {
        assert_eq!(aboard(at(altitude), &globe), at(altitude), "{altitude}");
    }
    for far in [top * 3.0, 1.0e16, f64::INFINITY, f64::NAN] {
        assert_eq!(aboard(at(far), &globe).altitude, top, "{far}");
    }
    assert_eq!(aboard(at(-20_000.0), &globe), at(ground + 1.5));
}
