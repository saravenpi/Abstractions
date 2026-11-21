//! Putting the ship in the places the courses start from: beside the home planet, and in a low orbit round it.

use bevy::math::{DQuat, DVec3};

use crate::ship::frames::planet_velocity;
use crate::ship::tests::rig::Game;
use crate::ship::warp::Warp;
use crate::space::{Clock, Frame};
use crate::universe::{BodyId, Universe};

/// Radii from the centre of the home planet the ship is put at at rest, outside its lock sphere.
pub const ORBIT: f64 = 3.0;

/// Radii from the centre of the home planet of a low circular orbit, inside its lock sphere.
const LOW: f64 = 1.1;

/// Puts the ship at rest [`ORBIT`] radii from the centre of the planet `body`, in the frame of its star.
pub fn beside(game: &mut Game, body: BodyId) {
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let planet = universe.planet(body);
    let up = DVec3::X;
    let pos = planet.position(time) + up * planet.radius * ORBIT;
    let frame = Frame::System(body.star);
    game.with_ship(|ship| {
        ship.frame = frame;
        ship.pos = pos;
        ship.rot = DQuat::from_rotation_arc(DVec3::NEG_Z, up);
        ship.vel = planet_velocity(&planet, time);
        (ship.spin, ship.throttle, ship.landed) = (DVec3::ZERO, 0.0, false);
        ship.warp = Warp::Off;
    });
    game.step(5);
}

/// Puts the ship at rest beside the home planet, [`ORBIT`] radii out, in the frame of the star.
pub fn into_orbit(game: &mut Game) {
    let home = game.get::<Universe>().home();
    beside(game, home);
}

/// Puts the ship in a circular orbit [`LOW`] radii from the centre of the home planet, nose along its
/// path, at the speed that holds it there: inside the lock sphere, where the drive is refused.
pub fn into_low_orbit(game: &mut Game) {
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let planet = universe.planet(universe.home());
    let (up, along) = (DVec3::X, DVec3::Z);
    let radius = planet.radius * LOW;
    let speed = (planet.mu() / radius).sqrt();
    let frame = Frame::System(universe.home().star);
    game.with_ship(|ship| {
        ship.frame = frame;
        ship.pos = planet.position(time) + up * radius;
        ship.rot = DQuat::from_rotation_arc(DVec3::NEG_Z, along);
        ship.vel = planet_velocity(&planet, time) + along * speed;
        (ship.spin, ship.throttle, ship.landed) = (DVec3::ZERO, 0.0, false);
        ship.warp = Warp::Off;
    });
    game.step(5);
}
