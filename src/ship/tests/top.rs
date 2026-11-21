//! The top speed of 0.11.0: twice the speed low over a planet, from a clean nose and a lever
//! that asks for more, within what the terrain streamer was tested at, and capped in thick air
//! by the thrust against the drag. The descent and the hover keep their old pace.

use bevy::math::DVec3;

use super::air::{air, world};
use super::hover::assisted;
use super::{fly, planet};
use crate::ship::cruise::{descent, top_speed};
use crate::ship::drag::{SEA_DENSITY, density, terminal};
use crate::ship::flight::{BOOST, Controls};

/// Fastest speed in metres per second the terrain streamer keeps the ground ahead ready at,
/// low over the ground: `world::chunks::tests_fly::fast` flies it.
const STREAMED: f64 = 300.0;

/// Returns the speed along the ground of an assisted ship that flew level for `seconds` with the
/// full lever, `height` metres over the sea of a planet with flat ground at the sea.
fn run(planet: &crate::universe::PlanetSpec, height: f64, seconds: f64) -> f64 {
    let mut ship = assisted(planet, height);
    ship.throttle = 1.0;
    let land = (planet, &(|_| 0.0) as &dyn Fn(DVec3) -> f64);
    fly(&mut ship, &Controls::default(), land, seconds, |_, _| {});
    ship.vel.length()
}

/// The nose first drag holds the engine to 275 m/s in air at sea level without the assist, twice
/// the 137 m/s of the drag area it had before, and a little more with the boost.
#[test]
fn the_clean_nose_doubles_the_top_speed_in_air() {
    let (now, before) = (terminal(SEA_DENSITY, 1.0), 137.3);
    assert!((now / before - 2.0).abs() < 0.01, "{now} m/s");
    assert!((terminal(SEA_DENSITY, BOOST) / now - BOOST.sqrt()).abs() < 1.0e-9);
    assert_eq!(terminal(0.0, 1.0), f64::INFINITY);
}

/// With the full lever an assisted ship 300 m over the sea of an Earth-sized planet flies at
/// about twice the 115 m/s it flew at before 0.11.0, and under the speed the terrain streamer
/// keeps up with.
#[test]
fn the_ship_flies_twice_as_fast_low_over_a_planet() {
    let earth = planet(6.0e6, 100_000.0, 86_400.0);
    let speed = run(&earth, 300.0, 8.0);
    let (asked, before) = (top_speed(300.0), 40.0 + 300.0 / 4.0);
    assert!(
        (speed / asked - 1.0).abs() < 0.02,
        "{speed} m/s for {asked}"
    );
    assert!(speed > 1.8 * before && speed < STREAMED, "{speed} m/s");
}

/// In air three times as thick as at home the lever asks for 415 m/s at 600 m, more than the
/// engine holds against the drag: the ship flies at the speed where they balance.
#[test]
fn thick_air_caps_the_top_speed() {
    let thick = world(air(50.0));
    let height = 600.0;
    let speed = run(&thick, height, 8.0);
    let cap = terminal(density(&thick, height), 1.0);
    assert!(cap < top_speed(height), "{cap} m/s");
    assert!(
        (speed / cap - 1.0).abs() < 0.03,
        "{speed} m/s against {cap}"
    );
}

/// The descent the assist allows near the ground is the one it allowed before the top speed
/// was raised: 1.8 m/s down to 3 m and a quarter of a metre per second for each metre above.
#[test]
fn the_descent_is_unchanged() {
    for clearance in [0.0, 3.0, 20.0, 150.0, 1000.0] {
        let before = 1.8 + (clearance - 3.0f64).max(0.0) / 4.0;
        assert!(
            (descent(clearance) - before).abs() < 1.0e-9,
            "{clearance} m"
        );
    }
}
