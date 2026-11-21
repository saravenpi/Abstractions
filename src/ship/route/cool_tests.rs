//! Tests of the speed of the climb: as fast as the pilot could fly at every height, under the
//! drag of the air and the heat cap through the air, so the nose glows only mildly, and fast
//! again above the air.

use bevy::math::DVec3;

use super::ascent::{Fix, altitude, order};
use super::ascent_tests::earth;
use crate::render::cool_speed;
use crate::ship::cruise::top_speed_at;
use crate::ship::drag::{density, heat_cap, terminal};
use crate::ship::tune::HEAT;

/// At every height of the climb the autopilot asks for the speed the full lever of the pilot
/// asks for there, under the drag and the heat cap; in the air it never asks for more than 1.4
/// times the speed at which the nose starts to glow, and above the air it flies the speed of
/// its orbit again.
#[test]
fn the_climb_flies_the_pilots_cap_under_the_heat_cap() {
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
        let air = density(&planet, height);
        let cap = top_speed_at(height, circular)
            .min(terminal(air, 1.0))
            .min(heat_cap(air));
        let asked = order(&fix, &planet, goal).throttle * cap;
        assert!(
            asked <= HEAT * cool_speed(air) * (1.0 + 1.0e-9),
            "{asked} m/s at {km} km"
        );
        if height < planet.atmosphere_height {
            assert!(
                (asked / cap - 1.0).abs() < 1.0e-9,
                "{asked} m/s at {km} km, cap {cap}"
            );
        } else {
            assert!(asked > 5000.0, "only {asked} m/s above the air at {km} km");
        }
    }
}
