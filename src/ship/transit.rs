//! Changing frame on the way: into and out of an atmosphere, a star system and a galaxy.

use bevy::math::DVec3;

use super::body::Ship;
use super::frames::{to_galaxy, to_star, to_surface, to_system};
use super::warp::Warp;
use super::wells::Well;
use crate::computer::SYSTEM_REACH;
use crate::space::Frame;
use crate::universe::{GalaxyId, PlanetSpec, StarSpec, SystemSpec, Universe};

/// Metres above the top of an atmosphere at which a climbing ship leaves the surface frame.
const AIR_MARGIN: f64 = 50.0;

/// Share of the reach of a system beyond which a ship leaves the frame of its star.
const REACH_MARGIN: f64 = 1.05;

/// Radii from a planet inside which it carries a ship across a gap in time.
const ESCORT: f64 = 100.0;

/// Moves a ship between the surface frame of a planet and the frame of its star: up above the
/// top of the atmosphere, down below it. A landed ship stays where it is.
pub fn reframe(ship: &mut Ship, system: &SystemSpec, time: f64) {
    match ship.frame {
        Frame::Surface(body) if !ship.landed => {
            let planet = system.planets.iter().find(|planet| planet.id == body);
            let top = |spec: &&PlanetSpec| {
                ship.pos.length() > spec.radius + spec.atmosphere_height + AIR_MARGIN
            };
            if let Some(planet) = planet.filter(top) {
                to_system(ship, planet, time);
            }
        }
        Frame::System(_) => {
            let inside = |spec: &&PlanetSpec| {
                spec.position(time).distance(ship.pos) < spec.radius + spec.atmosphere_height
            };
            if let Some(planet) = system.planets.iter().find(inside) {
                to_surface(ship, planet, time);
                ship.warp = Warp::Off;
            }
        }
        _ => {}
    }
}

/// Moves a ship from the frame of its star to the frame of its galaxy once it is out of reach.
pub fn leave_system(ship: &mut Ship, star: &StarSpec) {
    if ship.pos.length() > SYSTEM_REACH * REACH_MARGIN {
        to_galaxy(ship, star);
    }
}

/// Moves a ship between the stars into the galaxy whose grid cell it is in, when there is one.
pub fn change_galaxy(ship: &mut Ship, universe: &Universe, galaxy: GalaxyId) {
    let Some(old) = universe.galaxy(galaxy) else {
        return;
    };
    let at = old.locate(ship.pos);
    let Some(new) = universe
        .galaxy(GalaxyId(at.cell))
        .filter(|new| new.id != galaxy)
    else {
        return;
    };
    ship.pos = at - new.locate(DVec3::ZERO);
    ship.frame = Frame::Galaxy(new.id);
}

/// Moves a ship between the stars into the frame of the first of `stars` whose reach it is in.
pub fn enter_system<'a>(ship: &mut Ship, stars: impl IntoIterator<Item = &'a StarSpec>) {
    let near = |star: &&StarSpec| star.pos.distance(ship.pos) < SYSTEM_REACH;
    if let Some(star) = stars.into_iter().find(near) {
        to_star(ship, star);
    }
}

/// Carries a ship in the frame of a star across `gap` seconds in which it was not simulated,
/// such as a pause: a ship near a planet keeps its place and speed beside it.
pub fn catch_up(ship: &mut Ship, system: &SystemSpec, time: f64, gap: f64) {
    let near = |planet: &&PlanetSpec| {
        planet.position(time - gap).distance(ship.pos) < planet.radius * ESCORT
    };
    if let Some(planet) = system.planets.iter().find(near) {
        let (then, now) = (Well::planet(planet, time - gap), Well::planet(planet, time));
        ship.pos += now.pos - then.pos;
        ship.vel += now.vel - then.vel;
    }
}
