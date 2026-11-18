//! Warp optics tests: what speed does to directions, brightness and colour, and where a body
//! is seen through the lens.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::look::BETA_CAP;
use super::optics::{bend, bubble, doppler};
use super::shader::{aberrate, beaming, spread, tint};
use super::{CARRIED, Lens, Sequence};
use crate::render::scatter::luminance;

/// Speeds the rules are checked at, from rest to the cap.
const SPEEDS: [f32; 6] = [0.0, 0.1, 0.3, 0.5, 0.7, BETA_CAP];

/// Returns the lens of a ship flying along -Z at the speed `beta`.
fn lens(beta: f32) -> Lens {
    Lens {
        beta,
        ..Lens::default()
    }
}

/// At rest nothing moves or changes colour, and at any speed the points straight ahead and
/// straight behind stay where they are.
#[test]
fn rest_is_the_identity_and_the_poles_stay_put() {
    for step in 0..=40 {
        let cos = -1.0 + step as f32 / 20.0;
        assert_eq!((aberrate(cos, 0.0), doppler(cos, 0.0)), (cos, 1.0));
    }
    assert_eq!(
        (beaming(1.0), tint(1.0), spread(0.0)),
        (1.0, Vec3::ONE, 1.0)
    );
    let dir = DVec3::new(0.3, -0.5, 0.2).normalize();
    assert!((bend(dir, DVec3::NEG_Z, 0.0) - dir).length() < 1e-12);
    assert_eq!(lens(0.0).body(dir * 1.0e9, 1.0).offset, dir * 1.0e9);
    for beta in SPEEDS {
        assert_eq!((aberrate(1.0, beta), aberrate(-1.0, beta)), (1.0, -1.0));
        assert_eq!(bend(DVec3::NEG_Z, DVec3::NEG_Z, beta), DVec3::NEG_Z);
        assert_eq!(bend(DVec3::Z, DVec3::NEG_Z, beta), DVec3::Z);
        let seen = bend(dir, DVec3::NEG_Z, beta).dot(DVec3::NEG_Z);
        assert!((seen as f32 - aberrate(dir.dot(DVec3::NEG_Z) as f32, beta)).abs() < 1e-6);
    }
}

/// The sky crowds ahead more as the speed rises and never folds over; light is bluer and
/// brighter the nearer the heading it comes from, redder and dimmer behind.
#[test]
fn aberration_and_doppler_are_monotone() {
    for pair in SPEEDS.windows(2) {
        for step in 1..40 {
            let cos = -1.0 + step as f32 / 20.0;
            assert!(
                aberrate(cos, pair[1]) > aberrate(cos, pair[0]),
                "{cos} {pair:?}"
            );
            assert!((aberrate(aberrate(cos, pair[1]), -pair[1]) - cos).abs() < 1e-5);
        }
    }
    for beta in &SPEEDS[1..] {
        let seen: Vec<f32> = (0..=40).map(|i| -1.0 + i as f32 / 20.0).collect();
        for pair in seen.windows(2) {
            assert!(aberrate(pair[1], *beta) > aberrate(pair[0], *beta));
            assert!(doppler(pair[1], *beta) > doppler(pair[0], *beta));
            assert!(beaming(doppler(pair[1], *beta)) >= beaming(doppler(pair[0], *beta)));
        }
        assert!(doppler(1.0, *beta) > 1.0 && doppler(-1.0, *beta) < 1.0);
    }
}

/// At the cap the sky at right angles is seen at the cap's own angle, the Doppler factor has
/// its closed form, the light stays in its range and a full sequence shows exactly the cap.
#[test]
fn the_cap_bounds_everything() {
    assert_eq!(aberrate(0.0, BETA_CAP), BETA_CAP);
    let ahead = ((1.0 + BETA_CAP) / (1.0 - BETA_CAP)).sqrt();
    assert!((doppler(1.0, BETA_CAP) - ahead).abs() < 1e-4);
    assert!((doppler(-1.0, BETA_CAP) - 1.0 / ahead).abs() < 1e-4);
    assert!(beaming(doppler(1.0, BETA_CAP)) <= 3.0 && beaming(doppler(-1.0, BETA_CAP)) >= 0.08);
    assert!(beaming(1.0e9) <= 3.0 && beaming(0.0) >= 0.08);
    let full = Sequence {
        charge: 1.0,
        velocity: 1.0,
        ..Sequence::default()
    };
    assert_eq!(full.beta(), BETA_CAP);
    assert_eq!(Sequence::default().lens(Vec3::NEG_Z), Lens::default());
}

/// Light ahead turns blue and light behind turns red, while both tints stay near the
/// brightness of white: in black and white the shift is light and dark, not a new colour.
#[test]
fn the_colour_shift_keeps_its_brightness() {
    let (blue, red) = (tint(doppler(1.0, BETA_CAP)), tint(doppler(-1.0, BETA_CAP)));
    assert!(blue.z > blue.y && blue.y > blue.x, "{blue}");
    assert!(red.x > red.y && red.y > red.z, "{red}");
    assert!(tint(1.0e9).z > blue.z && tint(0.0).x > red.x);
    for factor in [0.2, 0.5, 0.9, 1.0, 1.1, 2.0, 5.0] {
        let light = luminance(tint(factor));
        assert!((0.85..=1.0).contains(&light), "{factor}: {light}");
    }
}

/// A streak is dimmer the longer it is, but its whole light never falls under that of the
/// star: stretching the stars does not make the sky flash or go dark.
#[test]
fn a_streak_spreads_the_light_of_its_star() {
    let halves: Vec<f32> = (0..=200).map(|i| i as f32 * 0.25).collect();
    let whole = |half: f32| spread(half) * (1.0 + half * 4.0 / std::f32::consts::PI);
    for pair in halves.windows(2) {
        assert!(spread(pair[1]) < spread(pair[0]) && whole(pair[1]) >= whole(pair[0]));
        assert!(spread(pair[0]) - spread(pair[1]) < 0.2);
    }
    assert!(spread(50.0) > 0.05 && whole(50.0) < 6.0);
}

/// A body in the warp bubble is seen as it is. In the open it is seen where a star in the
/// same direction is, smaller ahead by the Doppler factor, and nothing steps in between.
#[test]
fn a_body_keeps_its_place_among_the_stars() {
    let (lens, radius) = (lens(0.6), 6000.0);
    let dir = DVec3::new(0.5, 0.2, -0.7).normalize();
    let near = lens.body(dir * radius * 30.0, radius);
    assert_eq!((near.offset, near.scale), (dir * radius * 30.0, 1.0));
    let far = lens.body(dir * radius * 500.0, radius);
    let star = bend(dir, lens.heading.as_dvec3(), lens.beta);
    assert!((far.offset.normalize() - star).length() < 1e-9);
    assert!((far.offset.length() - radius * 500.0).abs() < 1.0);
    let factor = doppler(dir.as_vec3().dot(lens.heading), lens.beta);
    assert!(factor > 1.0 && (far.scale * factor - 1.0).abs() < 1e-5);
    assert_eq!((bubble(CARRIED.0), bubble(CARRIED.1)), (0.0, 1.0));
    let seen: Vec<_> = (300..=1100)
        .map(|i| lens.body(dir * radius * f64::from(i) * 0.1, radius))
        .collect();
    for pair in seen.windows(2) {
        let turn = pair[0].offset.angle_between(pair[1].offset);
        assert!(turn < 2e-3 && (pair[1].scale / pair[0].scale - 1.0).abs() < 2e-3);
    }
}

/// Bodies the ship passes slide aft, on their own side of the heading, the near ones faster
/// than the far ones, at rest and through the lens alike.
#[test]
fn bodies_slide_by_with_true_parallax() {
    let radius = 6.0e6;
    let aft = |lens: &Lens, side: f64, travelled: f64| {
        let offset = DVec3::new(side, 0.0, -2.0e9 + travelled);
        let seen = lens.body(offset, radius).offset;
        assert!(seen.x > 0.0 && seen.y.abs() < 1e-3 * seen.x, "{seen}");
        seen.angle_between(DVec3::NEG_Z)
    };
    for lens in [lens(0.0), lens(0.4), lens(BETA_CAP)] {
        let near: Vec<f64> = (0..=40)
            .map(|i| aft(&lens, 5.0e8, f64::from(i) * 1.0e8))
            .collect();
        let far: Vec<f64> = (0..=40)
            .map(|i| aft(&lens, 4.0e9, f64::from(i) * 1.0e8))
            .collect();
        assert!(near.windows(2).all(|pair| pair[1] > pair[0]), "{near:?}");
        assert!(far.windows(2).all(|pair| pair[1] > pair[0]), "{far:?}");
        assert!(near[20] - near[0] > 2.0 * (far[20] - far[0]));
        assert!(near[40] > std::f64::consts::FRAC_PI_2);
    }
}
