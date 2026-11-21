//! Tests of the height a saved place may have: what stays and what is put on the ground.

use bevy::math::DVec3;

use super::Globe;
use super::footing::grounded;
use crate::saves::Place;
use crate::universe::Universe;

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
        assert_eq!(grounded(at, &globe), at, "{lift}");
    }
}

/// A place deep under the ground, such as the lowest height a server repairs to, a height that
/// is not a number and a height beyond where the planet is drawn go to the ground in the same
/// direction.
#[test]
fn a_place_no_body_can_be_at_goes_to_the_ground() {
    let (globe, place, ground) = home();
    let beyond = globe.handover() - globe.radius() + 1.0;
    for altitude in [-20_000.0, ground - 40.0, f64::NAN, f64::INFINITY, beyond] {
        let fixed = grounded(Place { altitude, ..place }, &globe);
        assert_eq!((fixed.body, fixed.dir), (place.body, place.dir));
        let above = fixed.altitude - ground;
        assert!((1.0..2.0).contains(&above), "{altitude}: {above}");
        assert_eq!(grounded(fixed, &globe), fixed);
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
    assert!(grounded(lost, &globe).altitude.is_nan());
}
