//! Tests of the checks a saved place goes through before the height: a direction that points
//! somewhere, on a body that exists.

use super::model::Place;
use crate::universe::{BodyId, Universe};

/// A direction without a length or with a part that is not a number points nowhere.
#[test]
fn a_direction_without_a_length_points_nowhere() {
    let place = Place {
        dir: [0.0, 0.6, -0.8],
        ..Place::default()
    };
    assert!(place.pointed());
    for dir in [[0.0; 3], [f64::NAN, 0.0, 1.0], [f64::INFINITY, 0.0, 0.0]] {
        assert!(!Place { dir, ..place }.pointed(), "{dir:?}");
    }
}

/// The universe tells a body that exists from one that does not: a star, a planet or a moon
/// past the last one.
#[test]
fn the_universe_knows_which_bodies_exist() {
    let universe = Universe::new(4);
    let home = universe.home();
    assert!(universe.has(home));
    let planets = universe.system(home.star).planets.len() as u8;
    let past = |index: u8, moon: u8| BodyId {
        index,
        moon,
        ..home
    };
    assert!(universe.has(past(planets - 1, 0)));
    assert!(!universe.has(past(planets, 0)));
    assert!(!universe.has(past(home.index, 200)));
    let mut far = home;
    far.star.index = 250;
    assert!(!universe.has(far));
}
