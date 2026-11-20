//! Tests of the far galaxies and of the steps of a line of sight: a galaxy a few texels wide does
//! not flicker as the camera turns, and the steps run from the camera to the far edge.

use bevy::math::DVec3;

use super::fine::{cones, texel};
use super::march::{NEAR, SIDE, Source};
use super::rig::{home, texel as rig_texel};
use super::steps::{GROWTH, LEAST, STEP, segments};
use crate::universe::{Glow, LY};

/// Returns the light of the whole sky of a map of one galaxy `offset` metres away, texel by texel
/// on its share of the sphere, before the bytes round it: a galaxy this far gives under a byte.
fn glare(offset: DVec3) -> f64 {
    let (_, spec) = home();
    let far = [Source {
        glow: Glow::of(&spec),
        offset,
        inside: false,
    }];
    let fine = cones(&far, SIDE);
    let faces = (0..6).flat_map(|f| (0..SIDE).flat_map(move |r| (0..SIDE).map(move |c| (f, c, r))));
    faces
        .map(|at| {
            let (_, weight) = rig_texel(SIDE, at);
            let light = texel(&far, &fine, SIDE, at);
            f64::from(weight * (light.x + light.y + light.z) / 3.0)
        })
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

/// A line that starts at the camera has one segment up to the near sphere, then steps that
/// follow each other without a gap, each a quarter of its distance from the camera while that is
/// under the cap and the cap after, the last ending at the far edge, at most 96 in all.
#[test]
fn the_steps_grow_from_the_camera_to_the_far_edge() {
    let (near, far, radius) = (f64::from(NEAR) * LY, 7.0e4 * LY, 5.0e4 * LY);
    let all = segments(0.0, far, radius);
    assert_eq!(all[0], (0.0, near));
    assert!((all[1].0 / near - 1.0).abs() < 1e-12);
    assert!((all.last().map_or(0.0, |last| last.1) / far - 1.0).abs() < 1e-9);
    assert!(all.len() > LEAST && all.len() <= 96, "{} steps", all.len());
    for pair in all[1..].windows(2) {
        assert!((pair[1].0 / pair[0].1 - 1.0).abs() < 1e-9);
    }
    for (first, last) in &all[1..] {
        let step = last - first;
        assert!(step <= STEP * radius * (1.0 + 1e-9), "step {step}");
        assert!(step <= GROWTH * first * (1.0 + 1e-9) || step >= 0.5 * STEP * radius);
    }
    let steps: Vec<f64> = all[1..].iter().map(|(a, b)| b - a).collect();
    assert!(steps[1] > steps[0] && steps[0] < 0.2 * steps[steps.len() - 1]);
}

/// A line that meets a galaxy far outside the camera is cut in equal steps of at most the cap,
/// a short chord in the fewest steps, a line that ends inside the near sphere in one.
#[test]
fn a_far_line_is_cut_in_equal_steps_and_a_short_one_in_few() {
    let radius = 5.0e4 * LY;
    let from_afar = segments(1.0e6 * LY, 1.0e6 * LY + 1.4 * radius, radius);
    let widths: Vec<f64> = from_afar.iter().map(|(a, b)| b - a).collect();
    assert!(widths.iter().all(|w| (w / widths[0] - 1.0).abs() < 1e-9));
    assert!(widths[0] <= STEP * radius && from_afar.len() <= 96);
    let short = segments(1.0e6 * LY, 1.0e6 * LY + 0.01 * radius, radius);
    assert_eq!(short.len(), LEAST);
    assert_eq!(segments(0.0, 10.0 * LY, radius).len(), 1);
}
