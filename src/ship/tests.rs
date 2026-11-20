//! Ship tests: pure flight, ground and drive checks that need no window.

mod air;
mod airless;
mod aloft;
mod crash;
mod crossing;
mod cruise;
mod drive;
mod fx;
mod gearing;
mod giant;
mod ground;
mod handling;
mod helm;
mod hover;
mod hull;
mod landed;
mod leaving;
mod lever;
mod manual;
mod moon_route;
mod moons;
mod reach;
mod reroute;
mod resume;
mod retarget;
mod rig;
mod saved_frame;
mod space;
mod steady;
mod voyage;

use bevy::math::{DQuat, DVec3};

use super::body::Ship;
use super::flight::{Controls, Report, STEP, Scene, step};
use super::wells::Well;
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

/// Steps `ship` over `planet` for `seconds` without watching it.
fn coast(ship: &mut Ship, controls: &Controls, land: super::atmo::Land, seconds: f64) {
    fly(ship, controls, land, seconds, |_, _| {});
}

/// Steps `ship` twice among `wells`, which were sampled at the start of the first step.
fn two_steps(ship: &mut Ship, wells: &[Well]) {
    for sub in 0..2 {
        let scene = Scene {
            planet: None,
            ground: &|_| 0.0,
            wells,
            elapsed: f64::from(sub) * STEP,
        };
        step(ship, &Controls::default(), &scene, STEP);
    }
}
