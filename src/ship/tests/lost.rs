//! The saved ship that cannot come back, and the one in open space that has nothing to follow.

use bevy::math::{DQuat, DVec3};

use super::restore::{back, orbiting};
use crate::ship::body::{Ship, ShipState};
use crate::ship::restore::Restored;
use crate::space::Frame;
use crate::universe::Universe;

/// Universe time the records of these tests are checked at, in seconds.
const THEN: f64 = 5.0e5;

/// An empty ship inside a body, and a ship on a body or around a star that does not exist, are
/// lost: the world then lands a ship beside the player.
#[test]
fn a_ship_nobody_sits_in_is_lost_when_it_cannot_come_back() {
    let universe = Universe::new(7);
    let home = universe.home();
    let empty = Ship::at_rest(Frame::Surface(home), DVec3::Y * 10.0, DQuat::IDENTITY).state();
    assert_eq!(back(&universe, empty, THEN), Restored::Lost);
    let mut nowhere = home;
    nowhere.star.index = 250;
    let frames = [Frame::Surface(nowhere), Frame::System(nowhere.star)];
    for frame in frames {
        let state = ShipState {
            seated: true,
            ..Ship::at_rest(frame, DVec3::Y * 1.0e7, DQuat::IDENTITY).state()
        };
        assert_eq!(back(&universe, state, THEN), Restored::Lost, "{frame:?}");
    }
}

/// A record that is beside no body stays where it was against the star, however long the
/// clock ran.
#[test]
fn a_ship_in_open_space_stays_where_it_was() {
    let universe = Universe::new(7);
    let (planet, state, _) = orbiting(&universe);
    let alone = ShipState {
        beside: None,
        pos: (DVec3::Y * planet.orbit_radius).to_array(),
        ..state
    };
    let Restored::Kept(still, true) = back(&universe, alone, THEN + 86_400.0) else {
        panic!("open space was not kept");
    };
    assert_eq!(still.pos, DVec3::Y * planet.orbit_radius);
}
