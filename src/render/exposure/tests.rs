//! Tests of the metered exposure: it rises as the light falls, stays within its limits, eases
//! at the speeds of an eye, and lifts the unlit colours by the gain the lit things get.

use bevy::camera::Exposure;
use bevy::prelude::*;

use super::tune::{BRIGHTEN_LIMIT, DARKEN_LIMIT, REFERENCE_LUX};
use super::{Metered, eased, lifted, scene_lux, target_ev};
use crate::render::PlaceUniform;

/// A full day meters the reference, and every fall of the light opens the camera further, down to
/// its limit, which a moonless night reaches no further than.
#[test]
fn the_exposure_rises_as_the_light_falls_within_limits() {
    let reference = Exposure::EV100_BLENDER;
    assert!((target_ev(REFERENCE_LUX) - reference).abs() < 1.0e-5);
    let lux = [13_000.0, 8_000.0, 4_000.0, 2_000.0, 1_000.0, 300.0, 0.0];
    let evs = lux.map(target_ev);
    assert!(evs.windows(2).all(|pair| pair[1] <= pair[0]));
    assert!(evs[1] < evs[0] && evs[2] < evs[1] && evs[3] < evs[2]);
    assert!((evs[6] - (reference - BRIGHTEN_LIMIT)).abs() < 1.0e-5);
    assert!((target_ev(1.0e9) - (reference + DARKEN_LIMIT)).abs() < 1.0e-5);
}

/// The camera opens slowly and closes quickly, and never overshoots its target.
#[test]
fn the_eye_closes_faster_than_it_opens() {
    let closing = eased(8.7, 9.7, 1.0) - 8.7;
    let opening = 9.7 - eased(9.7, 8.7, 1.0);
    assert!(closing > opening && closing < 1.0 && opening > 0.0);
    assert_eq!(eased(9.0, 9.0, 1.0), 9.0);
    assert_eq!(eased(8.0, 9.0, 0.0), 8.0);
    assert!(eased(8.0, 9.0, 1.0e6) <= 9.0 + 1.0e-4);
}

/// The light of the place adds up as lux, and a place block of nothing meters nothing.
#[test]
fn the_scene_lux_adds_sun_moon_and_sky() {
    let place = PlaceUniform {
        eye_sun: Vec4::new(1_000.0, 1_000.0, 1_000.0, 0.0),
        eye_sky: Vec4::new(100.0, 100.0, 100.0, 0.0),
        ..default()
    };
    let lux = scene_lux(&place, 50.0);
    assert!((lux - (1_000.0 + 50.0 + std::f32::consts::PI * 100.0)).abs() < 0.5);
    assert_eq!(scene_lux(&PlaceUniform::default(), 0.0), 0.0);
}

/// The gain is a power of two of the stops under the reference, and the unlit colours double in
/// linear light for one stop, stay when the gain is one and never pass white.
#[test]
fn the_gain_lifts_unlit_colours_with_the_lit_ones() {
    let reference = Exposure::EV100_BLENDER;
    assert_eq!(Metered::default().gain(), 1.0);
    let bright = Metered {
        ev: reference - 1.0,
    };
    assert!((bright.gain() - 2.0).abs() < 1.0e-5);
    let dark = Vec3::new(0.05, 0.08, 0.18);
    assert_eq!(lifted(dark, 1.0), dark);
    let up = lifted(dark, 2.0);
    assert!(up.x > dark.x && up.y > dark.y && up.z > dark.z);
    let linear = |v: Vec3| Color::srgb(v.x, v.y, v.z).to_linear().red;
    assert!((linear(up) / linear(dark) - 2.0).abs() < 0.02);
    assert_eq!(lifted(Vec3::new(0.9, 0.9, 0.9), 50.0), Vec3::ONE);
}
