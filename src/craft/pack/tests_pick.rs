//! Tests of picking stones up: the stones of the planet underfoot, one after the other.

use super::pick::{handful, stones};
use crate::craft::matter::Stuff;
use crate::universe::Universe;

/// The home planet has stones, and the picks go round them, each a handful.
#[test]
fn picks_go_round_the_stones_of_the_planet() {
    let universe = Universe::new(1);
    let record = universe.planet(universe.home()).record;
    let found = stones(&record);
    assert!(!found.is_empty());
    for turn in 0..found.len() * 2 {
        let stack = handful(&record, turn).expect("a handful");
        assert_eq!(stack.stuff, Stuff::Mineral(found[turn % found.len()]));
        assert!(stack.mass > 0.0);
    }
}
