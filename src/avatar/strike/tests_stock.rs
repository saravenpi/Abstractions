//! Tests of the starter stock: the home planet of a universe grows a spear and a club from its own
//! life and stones.

use super::held::Style;
use super::stock::{on, stock};
use crate::craft::build::Intent;
use crate::life;
use crate::universe::Universe;
use crate::world::Globe;

/// Returns the globe of the home planet of universe 1, carrying its life.
fn home() -> Globe {
    let universe = Universe::new(1);
    let body = universe.home();
    let spec = universe.planet(body);
    let star = universe.system(body.star).star;
    Globe::of(&spec).with_life(life::of(&spec, &star))
}

/// The stock lists each tissue class once, and the solver makes a throwing spear and a club of it.
#[test]
fn the_home_planet_makes_a_spear_and_a_club() {
    let globe = home();
    let life = globe.life().expect("the home planet lives");
    let found = stock(life, globe.record(), globe.seed());
    assert!(!found.is_empty());
    let spear = on(&globe, Intent::Throw).expect("a spear");
    assert!(spear.thrown() && spear.style == Style::Thrust);
    let club = on(&globe, Intent::Crush).expect("a club");
    assert!(!club.thrown() && club.swing.energy > 0.0);
}
