//! Tests of the height the shaders measure from the camera: on a planet the size of the Earth the 32-bit
//! formula stays within a millimetre of the height worked out in 64 bits, where the distance to the
//! planet centre minus the radius is off by decimetres.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::track::block;
use super::{PlaceUniform, Sky};
use crate::render::air::Air;
use crate::render::observer::Observer;
use crate::render::sun::SunDir;
use crate::render::vantage::Vantage;

const SHADER: &str = include_str!("../daylight.wesl");

/// Radius in metres of the planet the tests stand on.
const RADIUS: f64 = 6.4e6;

/// Returns the block of a camera `high` metres above the sea of a planet of [`RADIUS`], at render
/// position `eye`, where `up` points away from the planet centre.
fn seen(eye: Vec3, up: DVec3, high: f64) -> PlaceUniform {
    let centre = (eye.as_dvec3() - up * (RADIUS + high)).as_vec3();
    let place = PlaceUniform {
        centre: centre.extend(RADIUS as f32),
        ..default()
    };
    place.seen_from(eye, up.as_vec3(), high)
}

/// Returns `count` render offsets from the camera within `reach` metres on each axis, spread by a fixed
/// rule.
fn offsets(count: u32, reach: f32) -> impl Iterator<Item = Vec3> {
    let unit =
        |n: u32| (n.wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
    (0..count).map(move |i| Vec3::new(unit(3 * i), unit(3 * i + 1), unit(3 * i + 2)) * reach)
}

/// Returns the largest error in metres of `height` over points within `reach` metres of a camera `high`
/// metres up, against the height worked out in 64 bits.
fn worst(high: f64, reach: f32, height: impl Fn(&PlaceUniform, Vec3) -> f32) -> f64 {
    let up = DVec3::new(0.3, 0.8, -0.52).normalize();
    let eye = Vec3::new(41.5, -7.25, 118.0);
    let place = seen(eye, up, high);
    let error = |p: Vec3| {
        let truth = (up * (RADIUS + high) + p.as_dvec3()).length() - RADIUS;
        (f64::from(height(&place, eye + p)) - truth).abs()
    };
    offsets(4000, reach).map(error).fold(0.0, f64::max)
}

/// Around a camera on the ground and one 300 m up, the formula is within a millimetre over 200 m and
/// within a centimetre over 3 km, where 32-bit render positions themselves are a quarter millimetre
/// apart. The distance to the centre minus the radius is off by more than a decimetre.
#[test]
fn the_formula_errs_under_a_millimetre_at_earth_size() {
    for high in [1.7, 300.0] {
        let near = worst(high, 200.0, PlaceUniform::altitude_at);
        assert!(near < 1.0e-3, "camera at {high} m: {near} m");
        let wide = worst(high, 3000.0, PlaceUniform::altitude_at);
        assert!(wide < 1.0e-2, "camera at {high} m: {wide} m");
    }
    let naive = |place: &PlaceUniform, world: Vec3| {
        (world - place.centre.truncate()).length() - place.centre.w
    };
    assert!(worst(1.7, 200.0, naive) > 0.1);
}

/// The block of the camera carries its render position, the way up and its height worked out in 64
/// bits, and the height of the camera itself reads back as that height.
#[test]
fn the_block_measures_from_the_camera() {
    let up = DVec3::new(-0.2, 0.6, 0.77).normalize();
    let observer = Observer {
        eye: DVec3::X * 1.5e11 + up * (RADIUS + 12.25),
        center: DVec3::X * 1.5e11,
        spin: DQuat::IDENTITY,
        to_render: DQuat::IDENTITY,
    };
    let vantage = Vantage {
        observer,
        camera: Vec3::new(3.0, 9.0, -4.0),
        up: up.as_vec3(),
        altitude: 12.25,
        radius: RADIUS as f32,
        ..default()
    };
    let sky = Sky::of(Vec3::ONE, &Air::HOME, (RADIUS as f32, 8.0e4), 0.0);
    let place = block(&sky, &vantage, &SunDir::default());
    assert_eq!(place.eye.truncate(), vantage.camera);
    assert!((place.eye.w - 12.25).abs() < 1.0e-3, "{}", place.eye.w);
    assert!((place.altitude_at(vantage.camera) - place.eye.w).abs() < 1.0e-3);
    let above = vantage.camera + up.as_vec3() * 40.0;
    assert!((place.altitude_at(above) - 52.25).abs() < 2.0e-3);
}

/// The shader holds the same formula, and the place of a point reads its height through it.
#[test]
fn the_shader_measures_as_the_block_does() {
    for line in [
        "let p = world - place.eye.xyz;",
        "let rise = dot(p, p) + 2.0 * (radius + place.eye.w) * dot(p, place.upward.xyz) + place.upward.w;",
        "return rise / max(sqrt(max(radius * radius + rise, 0.0)) + radius, 0.001);",
        "return vec2<f32>(dot(out / far, place.star.xyz), altitude_at(place, world));",
    ] {
        assert!(SHADER.contains(line), "{line}");
    }
    assert!(!SHADER.contains("far - place.centre.w"));
}
