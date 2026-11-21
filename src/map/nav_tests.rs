//! Map tests: the levels around each other, and where Back goes from each.

use bevy::prelude::*;

use super::{Look, Place, Stop, Trail};
use crate::computer::{Lens, Tab, Target};
use crate::map::level::Level;
use crate::universe::{BodyId, GalaxyId, StarId};

/// A galaxy, a star in it and a planet of that star.
pub(super) fn places() -> (GalaxyId, StarId, BodyId) {
    let galaxy = GalaxyId(IVec3::new(2, 0, -1));
    let star = StarId {
        galaxy,
        index: 3,
        ..default()
    };
    let body = BodyId {
        star,
        index: 1,
        moon: 0,
    };
    (galaxy, star, body)
}

/// Each level sits in the one around it up to the universe, and a double click on a target
/// goes into its own level.
#[test]
fn the_levels_sit_inside_each_other() {
    let (galaxy, star, body) = places();
    let path = [
        Place::Universe,
        Place::Galaxy(galaxy),
        Place::System(star),
        Place::Planet(body),
    ];
    assert_eq!(Place::Planet(body).path(), path);
    assert_eq!(Place::Universe.parent(), None);
    assert_eq!(Place::of(Target::Star(star)), Place::System(star));
    assert_eq!(Place::of(Target::Galaxy(galaxy)), Place::Galaxy(galaxy));
    let chart = (Tab::Galaxy, None);
    assert_eq!(
        Place::shown(Level::Chart, chart, star),
        Place::Galaxy(galaxy)
    );
    assert_eq!(
        Place::shown(Level::Planet(body), chart, star),
        Place::Planet(body)
    );
}

/// Back goes to the level shown before, or else to the level around this one, and nowhere
/// from the universe with nothing before.
#[test]
fn back_goes_to_the_level_before_or_around() {
    let (_, star, body) = places();
    let mut trail = Trail::default();
    let up = Stop {
        place: Place::System(star),
        lens: None,
    };
    assert_eq!(trail.back(Place::Planet(body)), Some(up));
    let before = Stop {
        place: Place::Universe,
        lens: Some(Look::Chart(Lens::default())),
    };
    trail.stops.push(before);
    assert_eq!(trail.back(Place::Planet(body)), Some(before));
    assert_eq!(trail.back(Place::Universe), None);
}
