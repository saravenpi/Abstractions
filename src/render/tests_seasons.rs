//! Tests of the season block the shaders read: the look of a point is the one of its own latitude and
//! height, the same from every camera, and the numbers of the shader are the numbers of the rules.

use std::f32::consts::FRAC_PI_4;

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::season::{AUTUMN_SHARE, FREEZE, GOLD, LEAF_GAIN, RUST, SNOW, SeasonLook};
use super::seasons::SeasonUniform;
use super::tests_season::{home, spot};
use crate::climate::{Setting, season as phase};
use crate::universe::PlanetSpec;

/// Returns the universe time at which `planet` is `season` of the way round its year.
fn when(planet: &PlanetSpec, season: f32) -> f64 {
    let ahead = (season - phase(planet, 0.0)).rem_euclid(1.0);
    planet.orbit_seconds * f64::from(ahead)
}

/// Returns the block of the home planet at `season` seen through the rotation `to_render`.
fn block(season: f32, to_render: DQuat) -> SeasonUniform {
    let planet = home();
    SeasonUniform::of(&planet, when(&planet, season), to_render)
}

/// The season of a point does not depend on the camera: the same place of the planet, turned into render
/// axes by any rotation, takes the same look at the same height, from the ground and from orbit.
#[test]
fn the_look_of_a_point_does_not_depend_on_the_camera() {
    let turned = DQuat::from_axis_angle(DVec3::new(0.3, 0.5, 0.8).normalize(), 1.9);
    let place = Vec3::new(0.6, 0.64, 0.48).normalize();
    for fraction in [0.0_f32, 0.2, 0.4, 0.7] {
        let (plain, other) = (block(fraction, DQuat::IDENTITY), block(fraction, turned));
        let there = (turned * place.as_dvec3()).as_vec3();
        for height in [0.0, 120.0, 4000.0] {
            let (a, b) = (plain.look(place, height), other.look(there, height));
            assert!((a.autumn - b.autumn).abs() < 1.0e-4, "{fraction} {height}");
            assert!((a.snow - b.snow).abs() < 1.0e-4, "{fraction} {height}");
        }
    }
}

/// The block gives the look the rules give: the autumn of a latitude at its season, and snow on a frozen
/// planet and none on a warm one, whatever height the camera looks from.
#[test]
fn the_block_gives_the_look_of_the_rules() {
    let planet = home();
    let setting = Setting::of(&planet);
    let here = spot(FRAC_PI_4, 0.0);
    let want = SeasonLook::at(&setting, &here, 0.4);
    let got = block(0.4, DQuat::IDENTITY).look(here.dir.as_vec3(), 0.0);
    assert!(
        (got.autumn - want.autumn).abs() < 1.0e-3,
        "{got:?} {want:?}"
    );
    assert!(got.autumn > 0.9, "{got:?}");
    let mut cold = planet.clone();
    cold.record.temperature = 250.0;
    let frozen = SeasonUniform::of(&cold, when(&cold, 0.4), DQuat::IDENTITY);
    assert_eq!(frozen.look(spot(1.5, 0.0).dir.as_vec3(), 0.0).snow, 1.0);
    let warm = block(0.4, DQuat::IDENTITY);
    assert_eq!(warm.look(spot(0.0, 0.0).dir.as_vec3(), 0.0).snow, 0.0);
}

/// A planet without a tilt has no autumn, one without air has no snow, and the empty block shows nothing.
#[test]
fn no_tilt_no_autumn_no_air_no_snow_and_no_block_no_season() {
    let mut flat = home();
    flat.record.tilt = 0.0;
    let time = when(&flat, 0.4);
    let level = SeasonUniform::of(&flat, time, DQuat::IDENTITY);
    assert_eq!(level.look(spot(0.8, 0.0).dir.as_vec3(), 0.0).autumn, 0.0);
    let mut bare = home();
    bare.record.temperature = 100.0;
    bare.record.pressure = 0.0;
    let airless = SeasonUniform::of(&bare, time, DQuat::IDENTITY);
    assert_eq!(airless.look(spot(0.8, 0.0).dir.as_vec3(), 0.0).snow, 0.0);
    let none = SeasonUniform::default().look(Vec3::X, 0.0);
    assert_eq!(none, SeasonLook::NONE);
}

/// The block is four texels of four floats, in the order `season_from` reads them.
#[test]
fn the_texels_follow_the_fields() {
    let block = block(0.3, DQuat::IDENTITY);
    let bytes = block.texels();
    assert_eq!(bytes.len(), 64);
    let numbers: Vec<f32> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();
    let fields: Vec<f32> = block.fields().iter().flat_map(Vec4::to_array).collect();
    assert_eq!(numbers, fields);
}

/// The constants of `season.wesl` are those of the rules in `season.rs`, so the tint on the GPU is the tint
/// the tests check on the CPU.
#[test]
fn the_shader_has_the_numbers_of_the_rules() {
    let shader = include_str!("season.wesl");
    let vec3 = |v: Vec3| format!("vec3<f32>({:?}, {:?}, {:?})", v.x, v.y, v.z);
    let wants = [
        format!(
            "const FREEZE: vec2<f32> = vec2<f32>({:?}, {:?});",
            FREEZE.0, FREEZE.1
        ),
        format!("const AUTUMN_SHARE: f32 = {AUTUMN_SHARE:?};"),
        format!("const LEAF_GAIN: f32 = {LEAF_GAIN:?};"),
        format!("const GOLD: vec3<f32> = {};", vec3(GOLD)),
        format!("const RUST: vec3<f32> = {};", vec3(RUST)),
        format!("const SNOW: vec3<f32> = {};", vec3(SNOW)),
    ];
    for want in wants {
        assert!(shader.contains(&want), "season.wesl lacks {want}");
    }
}
