//! Tests of the map seen from outside a galaxy and on the way in: a spiral has the shape its
//! tilt gives, and its brightness changes without a jump from far outside to deep inside.

use bevy::math::DVec3;

use super::march::{SIDE, Source, bake_at};
use super::rig::{home, texels};
use crate::universe::{GalaxySpec, Glow};

/// Returns the home galaxy alone as the camera `at` metres from its centre sees it.
fn lone(spec: &GalaxySpec, at: DVec3) -> Source {
    let offset = -at;
    Source {
        glow: Glow::of(spec),
        offset,
        inside: offset.length() <= spec.radius,
    }
}

/// A spiral seen from outside, its disc tilted 40 degrees from face on, is an ellipse whose
/// minor axis over its major axis is the cosine of that tilt, within a tenth.
#[test]
fn a_spiral_has_the_aspect_its_tilt_gives() {
    let (_, spec) = home();
    let tilt = 40.0f64.to_radians();
    let at = DVec3::new(tilt.sin(), tilt.cos(), 0.0) * 3.0 * spec.radius;
    let toward = (-at).normalize();
    let minor = (DVec3::Y - toward * toward.dot(DVec3::Y)).normalize();
    let major = toward.cross(minor);
    let bytes = bake_at(&[lone(&spec, at)], SIDE);
    let (mut across, mut along, mut total) = (0.0, 0.0, 0.0);
    for (dir, weight, light) in texels(&bytes, SIDE) {
        if dir.angle_between(toward) > 0.4 {
            continue;
        }
        let seen = f64::from(weight * light);
        let flat = dir - toward * dir.dot(toward);
        across += seen * flat.dot(minor).powi(2);
        along += seen * flat.dot(major).powi(2);
        total += seen;
    }
    assert!(total > 0.0, "the galaxy is dark");
    let aspect = (across / along).sqrt();
    assert!(
        (aspect / tilt.cos() - 1.0).abs() < 0.1,
        "aspect {aspect}, wanted {}",
        tilt.cos()
    );
}

/// Returns the light of the whole sky, texel by texel on its share of the sphere.
fn glare(bytes: &[u8]) -> f64 {
    texels(bytes, SIDE)
        .iter()
        .map(|(_, w, l)| f64::from(w * l))
        .sum()
}

/// Along a warp from three radii out to half a radius from the centre of the galaxy, past its
/// edge, the light of the whole sky changes by less than 5 percent between two steps a tiny
/// bit of the way apart, and grows as the camera comes in.
#[test]
fn the_light_changes_without_a_jump_from_outside_to_inside() {
    let (_, spec) = home();
    let toward = DVec3::new(0.8, 0.1, 0.6).normalize();
    let steps = 120;
    let at = |k: usize| toward * 3.0 * spec.radius * (0.5f64 / 3.0).powf(k as f64 / steps as f64);
    let lights: Vec<f64> = (0..=steps)
        .map(|k| glare(&bake_at(&[lone(&spec, at(k))], SIDE)))
        .collect();
    for (k, pair) in lights.windows(2).enumerate() {
        let change = (pair[1] / pair[0] - 1.0).abs();
        assert!(change < 0.05, "step {k}: {} to {}", pair[0], pair[1]);
    }
    assert!(lights[steps] > 2.0 * lights[0]);
}
