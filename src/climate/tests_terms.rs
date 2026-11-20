//! Tests of the terms of the temperature: summed as a shader sums them, they give the temperature of the
//! climate.

use bevy::math::{DVec3, Vec2, Vec3};

use super::tests::{homeworld, spot};
use super::{Spot, at, heat, terms};

/// The terms sum to the climate temperature to a hundredth of a kelvin, for a body that turns freely and
/// for a locked moon, at sea level and high up.
#[test]
fn the_terms_sum_to_the_temperature() {
    let mut setting = homeworld();
    for locked in [false, true] {
        setting.near = locked.then_some(Vec3::new(-0.6, 0.0, 0.8));
        for (lat, lon, altitude, season) in [(0.0, 0.0, 0.0, 0.25), (45.0, 80.0, 900.0, 0.7)] {
            let here = Spot {
                altitude,
                ..spot(lat, lon)
            };
            let full = at(&setting, &here, season).temperature;
            let sin = heat::latitude(&setting, here.dir.as_vec3()).sin();
            let sum = terms(&setting, season).at(sin, here.dir.as_vec3(), altitude);
            assert!((full - sum).abs() < 0.01, "{locked} {lat} {full} {sum}");
        }
    }
}

/// Without a tilt the season adds nothing, and without gravity height cools nothing.
#[test]
fn no_tilt_no_swing_and_no_gravity_no_lapse() {
    let mut setting = homeworld();
    setting.tilt = 0.0;
    assert_eq!(terms(&setting, 0.1).swing, 0.0);
    setting.gravity = 0.0;
    let flat = Spot {
        dir: DVec3::X,
        altitude: 0.0,
        inland: 0.0,
        slope: Vec2::ZERO,
    };
    let low = terms(&setting, 0.1).at(0.0, flat.dir.as_vec3(), 0.0);
    assert_eq!(
        terms(&setting, 0.1).at(0.0, flat.dir.as_vec3(), 5000.0),
        low
    );
}
