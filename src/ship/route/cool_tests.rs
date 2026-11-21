//! Tests of the speed of the climb through the air: slow where the air is thick, so that the
//! nose never glows, and fast again above the air.

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
