//! Frame changes: the same ship re-expressed on a turning planet, around its star or in its galaxy.

use std::f64::consts::TAU;

use bevy::math::DVec3;

use super::body::Ship;
use crate::space::Frame;
use crate::universe::{PlanetSpec, StarSpec};
use crate::world::Link;

/// Returns the angular speed of a planet around its +Y axis in radians per second.
pub fn spin_rate(planet: &PlanetSpec) -> f64 {
    TAU / planet.day_seconds
}

/// Returns the velocity of a planet in the system frame at `time` seconds, in metres per second.
pub fn planet_velocity(planet: &PlanetSpec, time: f64) -> DVec3 {
    planet.position(time + 0.5) - planet.position(time - 0.5)
}

/// Moves a ship from the surface frame of `planet` to the frame of its star.
///
/// The velocity gains the ground speed of the turning planet and its orbital velocity.
pub fn to_system(ship: &mut Ship, planet: &PlanetSpec, time: f64) {
    let link = Link::orbiting(planet, time);
    let omega = DVec3::Y * spin_rate(planet);
    ship.spin += ship.rot.inverse() * omega;
    ship.vel = planet_velocity(planet, time) + link.turn * (ship.vel + omega.cross(ship.pos));
    ship.pos = link.to_anchor(ship.pos);
    ship.rot = (link.turn * ship.rot).normalize();
    ship.frame = Frame::System(planet.id.star);
}

/// Moves a ship from the frame of the star of `planet` to the surface frame of `planet`.
pub fn to_surface(ship: &mut Ship, planet: &PlanetSpec, time: f64) {
    let link = Link::orbiting(planet, time);
    let back = link.turn.inverse();
    let omega = DVec3::Y * spin_rate(planet);
    ship.pos = link.to_planet(ship.pos);
    ship.vel = back * (ship.vel - planet_velocity(planet, time)) - omega.cross(ship.pos);
    ship.rot = (back * ship.rot).normalize();
    ship.spin -= ship.rot.inverse() * omega;
    ship.frame = Frame::Surface(planet.id);
}

/// Moves a ship from the frame of `star` to the frame of its galaxy. Stars do not move.
pub fn to_galaxy(ship: &mut Ship, star: &StarSpec) {
    ship.pos += star.pos;
    ship.frame = Frame::Galaxy(star.id.galaxy);
}

/// Moves a ship from the frame of the galaxy of `star` to the frame of `star`.
pub fn to_star(ship: &mut Ship, star: &StarSpec) {
    ship.pos -= star.pos;
    ship.frame = Frame::System(star.id);
}

/// Returns the Hill radius in metres of `planet` around what it goes round: `a (m / 3 M)^(1/3)`.
///
/// The gravitational parameter of what it goes round follows from its orbit, which is circular:
/// `mu = (2 pi / T)^2 a^3`.
pub fn hill_radius(planet: &PlanetSpec) -> f64 {
    let (a, rate) = (planet.orbit_radius, TAU / planet.orbit_seconds);
    a * (planet.mu() / (3.0 * rate * rate * a * a * a)).cbrt()
}
