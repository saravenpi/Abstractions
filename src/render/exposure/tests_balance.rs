//! Tests of the white balance: white light changes nothing, a red light moves the white partly
//! toward it, the degree follows CAT02, the balance eases slowly, and the white is the one of
//! noon at every hour.

use bevy::prelude::*;

use super::balance::{D65, adapted, chromaticity, degree, eased, grading};
use super::tune::SUN_LIGHT;
use crate::render::air::Air;
use crate::render::daylight::Sky;

/// The light of the Sun is the neutral light of the game: the balance is the identity.
#[test]
fn white_light_is_the_identity() {
    let white = adapted(SUN_LIGHT * 12_000.0, 12_000.0);
    assert!((white - D65).length() < 1.0e-3);
    let (temperature, tint) = grading(white);
    assert!(temperature.abs() < 1.0e-3 && tint.abs() < 1.0e-3);
}

/// A red light gives a white between D65 and the red, and a redder light gives a redder white.
#[test]
fn red_light_is_adapted_to_partly() {
    let red = Vec3::new(1.0, 0.45, 0.2) * 8_000.0;
    let (own, white) = (chromaticity(red / SUN_LIGHT), adapted(red, 8_000.0));
    assert!(own.x > D65.x + 0.05);
    assert!(white.x > D65.x && white.x < own.x);
    assert!((white.x - D65.x) > 0.9 * (own.x - D65.x));
    let (temperature, _) = grading(white);
    assert!(temperature < 0.0);
}

/// CAT02: a bright scene is nearly fully adapted to, a dark one less, and never beyond 1 or
/// under 0.82.
#[test]
fn the_degree_follows_cat02() {
    assert!(degree(12_000.0) > 0.99 && degree(12_000.0) <= 1.0);
    assert!(degree(0.0) > 0.82 && degree(0.0) < degree(100.0));
    assert!((degree(100.0) - 0.84).abs() < 0.02);
}

/// The white moves a minute's worth of seconds to within 10 percent of the target, and never
/// overshoots it.
#[test]
fn the_balance_eases_in_about_a_minute() {
    let target = Vec2::new(0.45, 0.38);
    let mut white = D65;
    for _ in 0..60 {
        white = eased(white, target, 1.0);
        assert!(white.x <= target.x);
    }
    assert!((target.x - white.x) < 0.1 * (target.x - D65.x));
}

/// The white is the light of noon at every hour: at noon it is the light the camera gets, so the
/// noon picture is as it was, and a low star, whose light is redder, keeps its gold against it.
#[test]
fn the_white_is_the_light_of_noon() {
    let star = Vec3::new(1.0, 0.96, 0.88);
    let sky = Sky::of(star, &Air::HOME, (6.371e6, 96_000.0), 60.0);
    let place = |height: f32| sky.uniform(Vec3::ZERO, Vec3::Y, (height, 0.0));
    let at_noon = place(1.0);
    let camera = at_noon.eye_sun.truncate() + std::f32::consts::PI * at_noon.eye_sky.truncate();
    assert!((at_noon.noon() - camera).abs().max_element() < 1.0e-2 * camera.max_element());
    let low = place(0.05);
    assert_eq!(low.noon(), at_noon.noon());
    let gold = chromaticity(low.eye_sun.truncate());
    assert!(gold.x > chromaticity(low.noon()).x + 0.03, "{gold}");
}
