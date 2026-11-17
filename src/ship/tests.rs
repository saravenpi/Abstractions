//! Ship tests: pure flight, ground and drive checks that need no window.

mod drive;
mod ground;
mod space;

use bevy::math::{DQuat, DVec3};

use super::body::Ship;
use super::flight::{Controls, Report, STEP, Scene, step};
use crate::space::Frame;
use crate::universe::{PlanetSpec, Universe};

/// Returns the home planet resized to `radius` metres with an atmosphere `air` metres high, turning
/// once every `day` seconds.
fn planet(radius: f64, air: f64, day: f64) -> PlanetSpec {
    let universe = Universe::new(7);
    PlanetSpec {
        radius,
        atmosphere_height: air,
        day_seconds: day,
        ..universe.planet(universe.home())
    }
}

/// Returns a ship without assist in the surface frame of `planet`, `height` metres above its sea at
/// the +Y pole, level with its nose along -Z.
fn above_pole(planet: &PlanetSpec, height: f64) -> Ship {
    let pos = DVec3::Y * (planet.radius + height);
    let mut ship = Ship::at_rest(Frame::Surface(planet.id), pos, DQuat::IDENTITY);
    ship.assist = false;
    ship
}

/// Steps `ship` over `planet` for `seconds`, calling `each` after every step.
fn fly(
    ship: &mut Ship,
    controls: &Controls,
    land: super::atmo::Land,
    seconds: f64,
    mut each: impl FnMut(&Ship, &Report),
) {
    let scene = Scene {
        planet: Some(land.0),
        ground: land.1,
        wells: &[],
        elapsed: 0.0,
    };
    for _ in 0..(seconds / STEP).round() as usize {
        let report = step(ship, controls, &scene, STEP);
        each(ship, &report);
    }
}
