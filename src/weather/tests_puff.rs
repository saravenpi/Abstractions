//! Tests that the Rust side of the fine cloud is the shader's twin: fixed points against values
//! worked out by hand from `render/cloud_field.wesl`, and the same constants in both files.

use bevy::math::Vec3;

use super::puff::{
    EDGE, OCTAVE_SCALE, OCTAVE_SHIFT, OCTAVES, THRESHOLD, density, fbm, hash31, space_noise,
    threshold,
};

/// The text of the shader module both cloud shells import.
const SHADER: &str = include_str!("../render/cloud_field.wesl");

/// Returns the numbers the shader gives its constant `name`, in order.
fn constant(name: &str) -> Vec<f32> {
    let line = SHADER
        .lines()
        .find(|line| line.starts_with(&format!("const {name}:")))
        .unwrap_or_else(|| panic!("the shader has no constant {name}"));
    let value = line.split('=').nth(1).unwrap_or_default();
    let digits = |c: char| c.is_ascii_digit() || c == '.' || c == '-';
    let tail = value.split_once('(').map_or(value, |(_, tail)| tail);
    tail.split(|c: char| !digits(c))
        .filter_map(|number| number.parse().ok())
        .collect()
}

/// The hash is whole-number math, so it matches the shader bit for bit on whole points.
#[test]
fn the_hash_matches_the_shader() {
    let points = [
        (Vec3::ZERO, 0.0),
        (Vec3::new(1.0, 2.0, 3.0), 0.877_080_86),
        (Vec3::new(-4.0, 7.0, -1.0), 0.567_238_2),
        (Vec3::new(1000.0, -2000.0, 3000.0), 0.936_602_3),
    ];
    for (point, value) in points {
        assert_eq!(hash31(point), value, "{point}");
    }
}

/// The smooth noise, its octaves and the puff cut from them match the shader on fixed points,
/// up to the rounding of 32-bit floats.
#[test]
fn the_noise_and_the_puffs_match_the_shader() {
    let near = |a: f32, b: f32| (a - b).abs() < 2.0e-5;
    let (a, b, c) = (
        Vec3::new(3.25, -1.75, 8.125),
        Vec3::new(-12.5, 4.0, 0.375),
        Vec3::new(20.0, 13.5, -7.25),
    );
    assert!(near(space_noise(Vec3::splat(0.5)), 0.342_043_56));
    assert!(near(space_noise(a), 0.490_168_23));
    assert!(near(space_noise(b), 0.490_336_8));
    assert!(near(fbm(a, 4), 0.405_665));
    assert!(near(fbm(b, 4), 0.525_343_83));
    assert!(near(fbm(c, 4), 0.554_901_6));
    assert_eq!(density(a, 0.5), 0.0);
    assert!((density(b, 0.9) - 0.873_890_4).abs() < 1.0e-3);
    assert_eq!((density(c, 1.0), density(c, 0.0)), (1.0, 0.0));
    assert!(near(threshold(0.0), 0.72) && near(threshold(1.0), 0.4));
}

/// The constants both sides share carry the same name and the same value.
#[test]
fn the_constants_match_the_shader() {
    assert_eq!(constant("EDGE"), [EDGE]);
    assert_eq!(constant("THRESHOLD"), [THRESHOLD.0, THRESHOLD.1]);
    assert_eq!(constant("OCTAVE_SCALE"), [OCTAVE_SCALE]);
    assert_eq!(constant("OCTAVE_SHIFT"), OCTAVE_SHIFT.to_array());
    assert_eq!(constant("OCTAVES"), [OCTAVES as f32]);
}

/// More cover means more cloud at every point, and a clear sky still holds a few thin puffs.
#[test]
fn more_cover_means_more_cloud() {
    let points = (0..400).map(|i| Vec3::new(0.37, 0.91, -0.53) * i as f32);
    let (mut thin, mut full) = (0.0, 0.0);
    for point in points {
        let levels = [0.0, 0.25, 0.5, 0.75, 1.0].map(|cover| density(point, cover));
        assert!(levels.windows(2).all(|pair| pair[0] <= pair[1]), "{point}");
        (thin, full) = (thin + levels[0], full + levels[4]);
    }
    assert!(thin / 400.0 < 0.05 && full / 400.0 > 0.5, "{thin} {full}");
}
