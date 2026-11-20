//! Tests of the far galaxies and of the steps of a line of sight: a galaxy a few texels wide does
//! not flicker as the camera turns, and the steps run from the camera to the far edge.

use bevy::math::DVec3;

use super::march::{NEAR, SIDE, Source, bake_at};
use super::rig::{home, texels};
use super::steps::{STEPS, segments};
use crate::universe::{Glow, LY};

/// Returns the light of the whole sky of a map, texel by texel on its share of the sphere.
fn glare(offset: DVec3) -> f64 {
    let (_, spec) = home();
    let far = Source {
        glow: Glow::of(&spec),
        offset,
        inside: false,
    };
    let bytes = bake_at(&[far], SIDE);
    texels(&bytes, SIDE)
        .iter()
        .map(|(_, w, l)| f64::from(w * l))
        .sum()
}

/// The home galaxy edge on, two and a half million light-years off, spans a few texels and is
/// thinner than one. Turning the camera by a third of a texel changes the light of the whole map
/// by less than 10 percent: one line of sight per texel would give or take the disc by chance.
#[test]
fn a_far_galaxy_does_not_flicker_as_the_camera_turns() {
    let distance = 2.5e6 * LY;
    let turn = 0.37 * std::f64::consts::FRAC_PI_2 / SIDE as f64;
    let first = glare(DVec3::X * distance);
    let second = glare(DVec3::new(distance, 0.0, turn * distance));
    assert!(first > 0.0, "the galaxy is dark");
    assert!(
        (second / first - 1.0).abs() < 0.1,
        "{first} against {second}"
    );
}

/// A line that starts at the camera has one segment up to the near sphere, then 24 that follow
/// each other, each longer than the last, the first starting at [`NEAR`] light-years and the last
/// ending at the far edge.
#[test]
fn the_steps_grow_from_the_camera_to_the_far_edge() {
    let (near, far) = (f64::from(NEAR) * LY, 1.0e5 * LY);
    let all: Vec<_> = segments(0.0, far).collect();
    assert_eq!(all.len(), STEPS + 1);
    assert_eq!(all[0], (0.0, near));
    assert!((all[1].0 / near - 1.0).abs() < 1e-12);
    assert!((all[STEPS].1 / far - 1.0).abs() < 1e-9);
    for pair in all[1..].windows(2) {
        assert!((pair[1].0 / pair[0].1 - 1.0).abs() < 1e-9);
        assert!(pair[1].1 - pair[1].0 > pair[0].1 - pair[0].0);
    }
    let from_afar: Vec<_> = segments(1.0e6 * LY, 1.1e6 * LY).collect();
    assert_eq!(from_afar.len(), STEPS);
    assert_eq!(from_afar[0].0, 1.0e6 * LY);
    assert!(segments(0.0, 10.0 * LY).count() == 1);
}
