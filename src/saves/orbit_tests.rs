//! Tests of the place of a player seated in the ship in space: it is where the ship is, against
//! the nearest planet, and it stays right while the planet goes round its star.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::capture::beside;
use super::model::WorldSave;
use crate::ship::{Ship, ShipState};
use crate::space::{Anchor, Frame};
use crate::universe::Universe;
use crate::world::Link;

/// Universe time the place is taken at, in seconds.
const THEN: f64 = 5.0e5;

/// A seated record has the player and the ship at one place: the place taken at the seat of a
/// ship in orbit resolves to the position of the ship, at its real height, and a day later it is
/// at the same height over the planet, wherever the planet has gone.
#[test]
fn a_seated_place_is_where_the_ship_is() {
    let universe = Universe::new(7);
    let planet = universe.planet(universe.home());
    let out = DVec3::new(0.4, 0.2, -0.9).normalize() * planet.radius * 1.3;
    let ship = planet.position(THEN) + out;
    let anchor = Anchor {
        frame: Frame::System(planet.id.star),
        origin: ship,
        rotation: DQuat::from_rotation_y(0.7),
        flat: false,
    };
    let body = (planet.id, planet.radius);
    let link = Link::orbiting(&planet, THEN);
    let place = beside(&anchor, &link, body, Vec3::ZERO).expect("a place in orbit");
    assert_eq!(place.body, planet.id);
    assert!((place.altitude - planet.radius * 0.3).abs() < 1.0);
    let spot = place.position(planet.radius).expect("a direction");
    assert!(link.to_anchor(spot).distance(ship) < 1.0);
    let later = THEN + 86_400.0;
    let moved = Link::orbiting(&planet, later).to_anchor(spot);
    let gap = moved.distance(planet.position(later));
    assert!((gap - planet.radius * 1.3).abs() < 1.0, "{gap}");
    let landed = Anchor {
        frame: Frame::Surface(planet.id),
        ..anchor
    };
    assert_eq!(beside(&landed, &Link::SAME, body, Vec3::ZERO), None);
}

/// A save knows whether the player sat in the ship from the record of the ship.
#[test]
fn a_save_says_whether_the_player_is_seated() {
    let ship = Ship::at_rest(
        Frame::Surface(Universe::new(7).home()),
        DVec3::Y,
        DQuat::IDENTITY,
    );
    let record = |seated: bool| {
        let state = ShipState {
            seated,
            ..ship.state()
        };
        Some(toml::Value::try_from(state).expect("a record"))
    };
    let save = |ship: Option<toml::Value>| WorldSave {
        ship,
        ..WorldSave::default()
    };
    assert!(save(record(true)).seated());
    assert!(!save(record(false)).seated());
    assert!(!save(None).seated());
}
