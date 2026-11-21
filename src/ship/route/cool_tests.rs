//! Tests of the speed of the climb through the air: slow where the air is thick, so that the
//! nose never glows, and fast again above the air; and the speed the assist flies in air.

use bevy::math::DVec3;

use super::ascent::{Fix, altitude, order};
use super::ascent_tests::earth;
use crate::render::cool_speed;
use crate::ship::cruise::top_speed_at;
use crate::ship::drag::density;

/// Through the air the climb never asks for a speed at which the nose would glow, and above the
/// air it asks for the speed of the orbit again.
#[test]
fn the_climb_keeps_the_nose_dark_in_the_air() {
    let planet = earth();
    let goal = altitude(&planet);
    for km in 1..140 {
        let height = f64::from(km) * 1000.0;
        let fix = Fix {
            pos: DVec3::X * (planet.radius + height),
            vel: DVec3::ZERO,
            frame: DVec3::ZERO,
        };
        let circular = (planet.mu() / fix.pos.length()).sqrt();
        let asked = order(&fix, &planet, goal).throttle * top_speed_at(height, circular);
        let cool = cool_speed(density(&planet, height));
        assert!(
            asked <= cool * (1.0 + 1.0e-9),
            "{asked} m/s at {km} km, over {cool}"
        );
        if height > planet.atmosphere_height {
            assert!(asked > 5000.0, "only {asked} m/s above the air at {km} km");
        }
    }
}

/// Returns the speed hold of a ship `clearance` metres up in air of `density`, nose level along
/// -Z, moving along it at `speed`, the lever full and the boost held.
fn held(density: f64, clearance: f64, speed: f64) -> crate::ship::cruise::Cruise {
    crate::ship::cruise::Cruise {
        vel: DVec3::NEG_Z * speed,
        nose: DVec3::NEG_Z,
        rise: DVec3::Y,
        up: DVec3::Y,
        throttle: 1.0,
        lift: 0.0,
        boost: true,
        brake: false,
        clearance,
        ground: false,
        grip: 1.0,
        orbital: 7800.0,
        air: density,
    }
}

/// Under full lever and boost the assist never asks for more than the speed at which the nose
/// starts to glow in the air around it, whatever the height, while a ship that is already
/// faster, falling in from orbit, is not braked to it by the engines.
#[test]
fn the_assist_keeps_powered_flight_dark() {
    for (density, clearance) in [
        (1.2, 300.0),
        (0.4, 5000.0),
        (0.0133, 1500.0),
        (1.0e-3, 30_000.0),
    ] {
        let asked = held(density, clearance, 180.0).target().length();
        assert!(
            asked <= cool_speed(density) * (1.0 + 1.0e-9),
            "{asked} at {density}"
        );
    }
    let falling = held(1.0e-4, 60_000.0, 7000.0).target().length();
    assert!(
        falling >= 7000.0 - 1.0e-6,
        "the assist brakes a fall from orbit to {falling}"
    );
}
