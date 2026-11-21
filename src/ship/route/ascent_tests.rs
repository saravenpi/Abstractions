//! Tests of the climb to orbit: the time it takes, the orbit it leaves and when a route is a climb.

use bevy::math::{DQuat, DVec3};

use super::ascent::{Fix, altitude, climbs, order};
use crate::ship::body::Ship;
use crate::ship::cruise::top_speed_at;
use crate::ship::flight::{Controls, STEP, Scene, step};
use crate::ship::orbit::apsides;
use crate::ship::steer::aim;
use crate::ship::tune::{ORBIT_ALTITUDE, ORBIT_TOLERANCE};
use crate::ship::wells::{PLANET_LOCK, Well};
use crate::space::Frame;
use crate::universe::{EARTH_RADIUS, PlanetSpec, STANDARD_GRAVITY, Universe};

/// Seconds the climb of the autopilot to a 200 km orbit may take.
const LIMIT: f64 = 180.0;

/// Returns a planet of the size and the gravity of the Earth, with the air ending at 100 km.
pub(super) fn earth() -> PlanetSpec {
    let universe = Universe::new(7);
    PlanetSpec {
        radius: EARTH_RADIUS,
        gravity: STANDARD_GRAVITY,
        atmosphere_height: 1.0e5,
        ..universe.planet(universe.home())
    }
}

/// Returns the planet as a body at the origin of a star frame that does not move.
fn body(planet: &PlanetSpec) -> Well {
    Well {
        pos: DVec3::ZERO,
        vel: DVec3::ZERO,
        mu: planet.mu(),
        radius: planet.radius,
        lock: planet.radius * PLANET_LOCK,
        hill: f64::INFINITY,
    }
}

/// Returns where a ship in the frame of a star is against a planet at its origin.
fn fix_of(ship: &Ship) -> Fix {
    Fix {
        pos: ship.pos,
        vel: ship.vel,
        frame: DVec3::ZERO,
    }
}

/// Flies a ship from 1 km over the equator of `planet` with the climb to the height `goal` alone
/// until the climb says the orbit is reached or `limit` seconds are over, and returns the ship,
/// the seconds and whether it arrived.
pub(super) fn climb_from(planet: &PlanetSpec, (goal, limit): (f64, f64)) -> (Ship, f64, bool) {
    let wells = [body(planet)];
    let scene = Scene {
        planet: None,
        ground: &|_| 0.0,
        wells: &wells,
        elapsed: 0.0,
    };
    let start = DVec3::X * (planet.radius + 1000.0);
    let mut ship = Ship::at_rest(Frame::System(planet.id.star), start, DQuat::IDENTITY);
    let (mut time, mut arrived) = (0.0, false);
    while time < limit && !arrived {
        let asked = order(&fix_of(&ship), planet, goal);
        arrived = asked.done;
        (ship.throttle, ship.assist) = (asked.throttle * f64::from(!arrived), !arrived);
        let turn = aim(ship.rot, asked.aim);
        step(
            &mut ship,
            &Controls {
                turn,
                ..Controls::default()
            },
            &scene,
            STEP,
        );
        time += STEP;
    }
    (ship, time, arrived)
}

/// A ship that hangs 1 km over the equator of an Earth reaches a circular orbit at 200 km in
/// under 180 seconds of flight, steered only by the climb, and the orbit it is left in has its
/// highest and lowest points within the tolerance of the height asked for.
#[test]
fn the_climb_reaches_a_200_km_orbit_in_under_three_minutes() {
    let planet = earth();
    let (ship, time, arrived) = climb_from(&planet, (altitude(&planet), LIMIT));
    assert!(arrived, "no orbit after {time} s");
    let goal = planet.radius + ORBIT_ALTITUDE;
    let (far, near) = apsides(ship.pos, ship.vel, planet.mu()).expect("a closed orbit");
    assert!((far - goal).abs() < ORBIT_TOLERANCE, "apoapsis {far}");
    assert!((near - goal).abs() < ORBIT_TOLERANCE, "periapsis {near}");
}

/// Far from a surface the speed hold flies the space cap of 30 km/s over an Earth, whose orbit
/// at 200 km runs at under 9 km/s, and twice the orbit speed near a giant whose orbit is faster
/// than half the cap. At the surface it asks for 40 m/s.
#[test]
fn the_top_speed_follows_the_local_orbit() {
    let planet = earth();
    let circular = (planet.mu() / (planet.radius + ORBIT_ALTITUDE)).sqrt();
    assert!((7.0e3..9.0e3).contains(&circular), "{circular}");
    assert_eq!(top_speed_at(1.0e9, circular), 30_000.0);
    assert_eq!(top_speed_at(1.0e9, 20_000.0), 40_000.0);
    assert_eq!(top_speed_at(0.0, circular), 40.0);
}

/// A ship in the air of a planet climbs for any route, whatever its target, and a landed ship or
/// a ship in space does not.
#[test]
fn a_ship_in_the_air_climbs_for_every_route() {
    let planet = earth();
    let air = Ship::at_rest(Frame::Surface(planet.id), DVec3::Y * 6.4e6, DQuat::IDENTITY);
    assert!(climbs(&air));
    let landed = Ship {
        landed: true,
        ..air.clone()
    };
    assert!(!climbs(&landed));
    let space = Ship {
        frame: Frame::System(planet.id.star),
        ..air
    };
    assert!(!climbs(&space));
    assert_eq!(altitude(&planet), ORBIT_ALTITUDE);
    let thick = PlanetSpec {
        atmosphere_height: 3.0e5,
        ..planet
    };
    assert_eq!(altitude(&thick), 3.75e5);
}

/// The lock sphere of an Earth is 1.2 radii wide and sits well inside its Hill sphere, so a
/// warp locks near the planet and not over millions of kilometres.
#[test]
fn the_lock_is_a_little_outside_the_surface() {
    let planet = earth();
    let well = Well::planet(&planet, 0.0);
    assert_eq!(well.lock, 1.2 * planet.radius);
    assert!(well.hill > 5.0 * well.lock, "hill {}", well.hill);
}
