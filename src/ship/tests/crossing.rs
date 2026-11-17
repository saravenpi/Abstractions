//! Crossing the atmosphere top: the ship changes frame and nothing on screen may move.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::above_pole;
use crate::render::{Observer, SunDir};
use crate::ship::body::Ship;
use crate::ship::frames::{spin_rate, to_surface, to_system};
use crate::ship::mooring::anchor_for;
use crate::space::{Anchor, Frame};
use crate::universe::{PlanetSpec, StarSpec, Universe};
use crate::world::{Link, spin};

/// Returns the home planet and its star.
fn home() -> (PlanetSpec, StarSpec) {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let planet = system.planets[usize::from(universe.home().index)].clone();
    (planet, system.star)
}

/// What must not change when the ship changes frame.
struct Seen {
    /// The anchor the ship is drawn with.
    anchor: Anchor,
    /// Unit vector toward the star in render axes.
    sun: Vec3,
    /// Position of the ship in the frame of the planet.
    planet: DVec3,
    /// Nose of the ship in render axes.
    nose: Vec3,
}

/// Returns what is seen from `ship` at `time`, with the anchor that follows `before`.
fn seen(ship: &Ship, before: &Anchor, land: (&PlanetSpec, &StarSpec), time: f64) -> Seen {
    let (planet, star) = land;
    let anchor = anchor_for(before, (ship.frame, ship.pos), Some(planet), time);
    let eye = anchor.to_render(ship.pos);
    let observer = Observer::new(&anchor, eye, planet, star, time);
    let link = match ship.frame {
        Frame::Surface(_) => Link::SAME,
        _ => Link::orbiting(planet, time),
    };
    Seen {
        anchor,
        sun: SunDir::of(&observer, planet.radius).dir,
        planet: link.to_planet(ship.pos),
        nose: anchor.rotation_to_render(ship.rot) * Vec3::NEG_Z,
    }
}

/// Asserts that two views of the same moment agree: the sun within 1e-6, the ship within 1 mm.
fn assert_same(a: &Seen, b: &Seen, what: &str) {
    let sun = (a.sun - b.sun).length();
    let moved = (a.planet - b.planet).length();
    assert!(sun < 1e-6, "{what}: the sun turned by {sun}");
    assert!(moved < 1e-3, "{what}: the ship moved by {moved} m");
    assert!((a.nose - b.nose).length() < 1e-6, "{what}: the nose turned");
}

/// Leaving the surface frame above the atmosphere and coming back to it changes neither the
/// sun direction in render axes nor the place of the ship on the planet, at any time of day.
#[test]
fn crossing_the_atmosphere_top_moves_nothing() {
    let (planet, star) = home();
    let top = planet.radius + planet.atmosphere_height + 80.0;
    for time in [0.0, 777.25, 54_321.5, 9_876_543.0] {
        let mut ship = above_pole(&planet, 0.0);
        ship.pos = DVec3::new(0.48, 0.6, -0.64) * top;
        ship.rot = DQuat::from_euler(EulerRot::YXZ, 0.7, -0.3, 1.9);
        ship.vel = DVec3::new(40.0, 120.0, -15.0);
        let unset = Anchor {
            frame: Frame::System(star.id),
            origin: DVec3::ZERO,
            rotation: DQuat::IDENTITY,
            flat: false,
        };
        let below = seen(&ship, &unset, (&planet, &star), time);
        assert!((below.anchor.rotation * ship.pos.normalize() - DVec3::Y).length() < 1e-6);
        to_system(&mut ship, &planet, time);
        let above = seen(&ship, &below.anchor, (&planet, &star), time);
        assert_eq!(above.anchor.frame, Frame::System(star.id));
        assert_same(&below, &above, "going up");
        to_surface(&mut ship, &planet, time);
        let back = seen(&ship, &above.anchor, (&planet, &star), time);
        assert_eq!(back.anchor.frame, Frame::Surface(planet.id));
        assert_same(&below, &back, "coming back");
        assert_same(&above, &back, "coming down");
    }
}

/// The turn rate the flight uses is the rate of the one spin convention.
#[test]
fn spin_rate_is_the_rate_of_the_world_spin() {
    let (planet, _) = home();
    let (time, h) = (4321.0, 0.5);
    let turned = spin(planet.day_seconds, time).inverse() * spin(planet.day_seconds, time + h);
    let (axis, angle) = turned.to_axis_angle();
    assert!((axis - DVec3::Y).length() < 1e-9);
    assert!((angle / h - spin_rate(&planet)).abs() < 1e-12);
}
