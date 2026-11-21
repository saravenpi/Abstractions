//! The saved ship against the universe as it is now: an orbit comes back as it was and follows
//! its body, and a ship that would come back inside a planet, a gas giant or a star is put on
//! a safe circular orbit with the player in it.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::{Ship, ShipState};
use crate::ship::frames::planet_velocity;
use crate::ship::moons::with_moons;
use crate::ship::restore::{Restored, STAR_SAFE, Sky, restore};
use crate::ship::save::beside;
use crate::ship::warp::Warp;
use crate::space::Frame;
use crate::universe::{PlanetClass, PlanetSpec, Universe};
use crate::world::{Globe, Link};

/// Universe time the records of these tests are taken at, in seconds.
const THEN: f64 = 5.0e5;

/// Returns the home planet of seed 7 and the record of a ship seated on a circular orbit two
/// heights of its air up, taken at [`THEN`], with the ship itself.
pub(super) fn orbiting(universe: &Universe) -> (PlanetSpec, ShipState, Ship) {
    let planet = universe.planet(universe.home());
    let high = planet.radius + planet.atmosphere_height * 2.0;
    let speed = (planet.mu() / high).sqrt();
    let ship = Ship {
        vel: planet_velocity(&planet, THEN) + DVec3::NEG_Z * speed,
        ..Ship::at_rest(
            Frame::System(planet.id.star),
            planet.position(THEN) + DVec3::X * high,
            DQuat::IDENTITY,
        )
    };
    let system = with_moons(universe, universe.system(planet.id.star));
    let seated = ShipState {
        seated: true,
        ..ship.state()
    };
    let state = ShipState {
        beside: beside(&seated, &system, THEN),
        ..seated
    };
    (planet, state, ship)
}

/// Returns what becomes of `state` at `time` seconds in `universe`, the world starting at home.
pub(super) fn back(universe: &Universe, state: ShipState, time: f64) -> Restored {
    let globe = Globe::of(&universe.planet(universe.home()));
    let land = (universe.home(), &globe);
    restore(
        state,
        &Sky {
            universe,
            time,
            land,
        },
    )
}

/// Asserts that `restored` is a ship on a circular orbit around the body at `centre` moving at
/// `drift`, of parameter `mu`, farther than `floor` metres from it, in the frame of its star.
fn assert_orbit(restored: Restored, (centre, drift): (DVec3, DVec3), mu: f64, floor: f64) -> Ship {
    let Restored::Orbit(ship) = restored else {
        panic!("not put on an orbit: {restored:?}");
    };
    let (out, vel) = (ship.pos - centre, ship.vel - drift);
    let gap = out.length();
    assert!(gap > floor, "{gap} m is not above {floor} m");
    let round = (mu / gap).sqrt();
    assert!((vel.length() - round).abs() < round * 1e-6 + 1e-3);
    assert!(vel.dot(out).abs() < gap * round * 1e-6, "not circular");
    assert!(matches!(ship.frame, Frame::System(_)) && !ship.landed);
    assert_eq!((ship.warp, ship.throttle), (Warp::Off, 0.0));
    ship
}

/// A ship saved in low orbit comes back in the same orbit with its velocity, bit for bit, when
/// the clock did not move, and the record says which body it is beside.
#[test]
fn a_low_orbit_comes_back_as_it_was() {
    let universe = Universe::new(7);
    let (planet, state, ship) = orbiting(&universe);
    let held = state.beside.expect("beside the home planet");
    assert_eq!(held.body, planet.id);
    let high = planet.radius + planet.atmosphere_height * 2.0;
    assert!((DVec3::from(held.pos).length() - high).abs() < 1e-3);
    assert_eq!(back(&universe, state, THEN), Restored::Kept(ship, true));
}

/// A day later the planet has gone a long way round its star, and the ship comes back on the
/// same orbit beside it: the same position and velocity against the planet.
#[test]
fn a_ship_beside_a_body_follows_it_after_a_day() {
    let universe = Universe::new(7);
    let (planet, state, ship) = orbiting(&universe);
    let later = THEN + 86_400.0;
    let moved = planet.position(later).distance(planet.position(THEN));
    assert!(moved > planet.radius * 10.0, "the planet moved {moved} m");
    let Restored::Kept(after, true) = back(&universe, state, later) else {
        panic!("the orbit was not kept");
    };
    let was = (
        ship.pos - planet.position(THEN),
        ship.vel - planet_velocity(&planet, THEN),
    );
    let now = (
        after.pos - planet.position(later),
        after.vel - planet_velocity(&planet, later),
    );
    assert!(
        was.0.distance(now.0) < 1.0,
        "{} m off",
        was.0.distance(now.0)
    );
    assert!(was.1.distance(now.1) < 0.01);
}

/// A seated ship saved under the ground of a planet comes back on a circular orbit above its
/// air, in the same direction from its centre.
#[test]
fn a_ship_inside_a_planet_comes_back_in_orbit() {
    let universe = Universe::new(7);
    let planet = universe.planet(universe.home());
    let dir = DVec3::new(0.3, 0.9, 0.1).normalize();
    let sunk = Ship::at_rest(
        Frame::Surface(planet.id),
        dir * (planet.radius - 5.0e4),
        DQuat::IDENTITY,
    );
    let state = ShipState {
        seated: true,
        ..sunk.state()
    };
    let centre = (planet.position(THEN), planet_velocity(&planet, THEN));
    let floor = planet.radius + planet.atmosphere_height;
    let ship = assert_orbit(back(&universe, state, THEN), centre, planet.mu(), floor);
    let out = (ship.pos - centre.0).normalize();
    assert!(out.dot(Link::orbiting(&planet, THEN).turn * dir) > 1.0 - 1e-9);
}

/// A seated ship saved under the deck of a gas giant, in the frame of the giant or of its star,
/// comes back on a circular orbit above its air.
#[test]
fn a_ship_inside_a_gas_giant_comes_back_in_orbit() {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let gas = |planet: &&PlanetSpec| planet.record.class == PlanetClass::GasGiant;
    let giant = system.planets.iter().find(gas).expect("a gas giant");
    let deep = DVec3::Z * giant.radius * 0.5;
    let frames = [
        (Frame::Surface(giant.id), deep),
        (Frame::System(giant.id.star), giant.position(THEN) + deep),
    ];
    for (frame, pos) in frames {
        let state = ShipState {
            seated: true,
            ..Ship::at_rest(frame, pos, DQuat::IDENTITY).state()
        };
        let centre = (giant.position(THEN), planet_velocity(giant, THEN));
        let floor = giant.radius + giant.atmosphere_height;
        assert_orbit(back(&universe, state, THEN), centre, giant.mu(), floor);
    }
}

/// A seated ship saved inside a star, or too close to it, comes back on a circular orbit at a
/// safe distance, outside its warp lock.
#[test]
fn a_ship_inside_a_star_comes_back_in_orbit() {
    let universe = Universe::new(7);
    let star = universe.star(universe.home().star).expect("the home star");
    for share in [0.0, 0.5, STAR_SAFE - 0.1] {
        let pos = DVec3::new(0.6, 0.0, -0.8) * star.radius * share;
        let state = ShipState {
            seated: true,
            ..Ship::at_rest(Frame::System(star.id), pos, DQuat::IDENTITY).state()
        };
        let floor = star.radius * STAR_SAFE;
        assert_orbit(
            back(&universe, state, THEN),
            (DVec3::ZERO, DVec3::ZERO),
            star.mu(),
            floor,
        );
    }
}
