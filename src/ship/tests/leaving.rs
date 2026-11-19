//! Leaving the seat stops the motors: the throttle drops to idle, a spooling warp drive stops,
//! the autopilot lets go, and the landed ship stays where it stands. Boarding again finds the
//! motors as they were left, off.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::rig::Game;
use crate::computer::Target;
use crate::player::place;
use crate::settings::{Persistent, Settings};
use crate::ship::pilot::Autopilot;
use crate::ship::warp::Warp;
use crate::space::{Anchor, Location, Placed, Travel};
use crate::universe::Universe;

/// Taps the board key, which also leaves the seat, and runs two frames more.
fn tap_board(game: &mut Game) {
    let key = game
        .app
        .world()
        .resource::<Persistent<Settings>>()
        .controls
        .keys
        .ship
        .board;
    game.press(key);
    game.step(1);
    game.release(key);
    game.step(2);
}

/// Stands the player beside the hatch of the landed ship and takes the seat.
fn boards(game: &mut Game) {
    let anchor = game.get::<Anchor>();
    let world = game.app.world_mut();
    let mut ships = world.query_filtered::<&Placed, With<crate::ship::Ship>>();
    let hull = *ships.single(world).expect("one ship");
    let beside = hull.pos + hull.rot * DVec3::new(-2.6, -1.2, -1.8);
    let (feet, toward) = (anchor.to_render(beside), anchor.to_render(hull.pos));
    place(world, feet, toward - feet);
    game.step(2);
    tap_board(game);
    assert_eq!(game.get::<Location>().travel, Travel::InShip);
}

/// A pilot who leaves the landed ship with the throttle full, the warp drive spooling and a
/// route engaged leaves it with the throttle at idle, the drive off and no route. For three
/// seconds after, the ship gains no speed and stays landed. Back in the seat, the motors are
/// still off: boarding does not start them.
#[test]
fn leaving_the_seat_stops_the_motors() {
    let mut game = Game::full("leaving");
    game.start("Leaving", None);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    boards(&mut game);
    let home = game.get::<Universe>().home();
    game.app
        .world_mut()
        .resource_mut::<Autopilot>()
        .engage(Target::Body(home));
    game.with_ship(|ship| (ship.throttle, ship.warp) = (1.0, Warp::Spooling { left: 5.0 }));
    tap_board(&mut game);
    assert_eq!(game.get::<Location>().travel, Travel::OnFoot);
    let left = game.ship();
    assert_eq!((left.throttle, left.warp), (0.0, Warp::Off));
    assert_eq!(game.get::<Autopilot>().target(), None);
    for _ in 0..90 {
        game.step(1);
        let ship = game.ship();
        assert!(ship.landed && ship.throttle == 0.0 && ship.warp == Warp::Off);
        assert!(
            ship.vel.length() <= left.vel.length() + 1e-9,
            "{}",
            ship.vel
        );
    }
    boards(&mut game);
    game.step(10);
    assert_eq!(game.ship().throttle, 0.0);
}
