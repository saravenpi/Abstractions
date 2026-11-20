//! Shared setups of the guide tests: the lives of real home planets.

use crate::life::{self, Life};
use crate::universe::Universe;

/// Returns the life of the home planet of universe `seed`, or nothing when it is dead.
pub fn home(seed: u64) -> Option<Life> {
    let universe = Universe::new(seed);
    let body = universe.home();
    life::of(&universe.planet(body), &universe.system(body.star).star)
}

/// Returns the lives of the living home planets among the first `count` universes.
pub fn lives(count: u64) -> Vec<Life> {
    (0..count).filter_map(home).collect()
}

/// Returns the first living home planet with at least one animal that eats another.
pub fn web() -> Life {
    let rich = |l: &Life| l.species.iter().filter(|s| !s.prey.is_empty()).count() > 3;
    (0..64)
        .filter_map(home)
        .find(rich)
        .expect("a home planet with a food web")
}
