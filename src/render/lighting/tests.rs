//! Tests that the sun light is the light the ground shader takes as its reference at the camera,
//! and that the moon casts a shadow only when its light is strong enough to show one.

use bevy::light::DirectionalLight;
use bevy::prelude::*;

use super::tune::MOON_SHADOW_LUX;
use super::{Aim, SHADOW_HEIGHT, beam, sun_light};
use crate::render::{Palette, PlaceUniform, SunDir};

/// Returns a place block that is live and gives the camera the sun light `eye`.
fn place(eye: Vec3) -> PlaceUniform {
    PlaceUniform {
        air: Vec4::new(60_000.0, 60.0, 1.0, 0.0),
        eye_sun: eye.extend(0.0),
        ..default()
    }
}

/// Returns the light a light entity of this colour and lux gives, as a linear colour.
fn emitted(color: Color, lux: f32) -> Vec3 {
    let linear = LinearRgba::from(color);
    Vec3::new(linear.red, linear.green, linear.blue) * lux
}

/// Under a live place block the sun light is exactly the light the block gives the camera, so the
/// correction of the ground shader, which is the rule at a point less the block at the camera,
/// is zero at the camera. The palette does not enter.
#[test]
fn the_sun_gives_what_the_place_gives_the_camera() {
    let eye = Vec3::new(11_400.0, 10_100.0, 8_300.0);
    let sun = SunDir {
        height: 0.6,
        lit: 1.0,
        ..default()
    };
    let (color, lux) = sun_light(&place(eye), &sun, &Palette::at(0.3));
    let given = emitted(color, lux);
    assert!((given - eye).abs().max_element() < 0.05);
}

/// Without a live place block, as in space, the sun keeps the light of the palette.
#[test]
fn without_a_place_the_palette_lights() {
    let mut palette = Palette::at(0.3);
    palette.sun_lux = 7_000.0;
    let (_, lux) = sun_light(&PlaceUniform::default(), &SunDir::default(), &palette);
    assert_eq!(lux, 7_000.0);
}

/// An eclipse or a far star takes its share from the light of the star above the horizon, and
/// below the horizon the planet's shadow already is in the block, so nothing is taken twice.
#[test]
fn the_beam_is_taken_once() {
    let up = SunDir {
        height: 0.4,
        lit: 0.25,
        ..default()
    };
    let down = SunDir {
        height: -0.4,
        lit: 0.25,
        ..default()
    };
    assert_eq!(beam(&up), 0.25);
    assert_eq!(beam(&down), 1.0);
}

/// A sun that gives no light gives no shadow and no colour to divide by zero.
#[test]
fn a_dark_sun_is_white_and_zero() {
    let (color, lux) = sun_light(&place(Vec3::ZERO), &SunDir::default(), &Palette::at(0.3));
    assert_eq!(lux, 0.0);
    assert_eq!(emitted(color, lux), Vec3::ZERO);
}

/// Returns whether a light aimed with `floor` lux at `height` and `lux` casts shadows.
fn casts(height: f32, lux: f32, floor: f32) -> bool {
    let aim = Aim {
        dir: Vec3::Y,
        height,
        color: Color::WHITE,
        lux,
        shadows: true,
        floor,
    };
    let mut transform = Transform::default();
    let mut global = GlobalTransform::default();
    let mut light = DirectionalLight::default();
    aim.apply((&mut transform, &mut global), &mut light);
    assert_eq!(light.illuminance, lux);
    assert_eq!(global, GlobalTransform::from(transform));
    light.shadow_maps_enabled
}

/// The moon casts shadows only above its lux floor and above the horizon, the sun whenever it
/// gives light above the horizon.
#[test]
fn the_moon_casts_shadows_only_when_it_is_bright() {
    assert!(!casts(0.5, MOON_SHADOW_LUX * 0.9, MOON_SHADOW_LUX));
    assert!(!casts(0.5, MOON_SHADOW_LUX, MOON_SHADOW_LUX));
    assert!(casts(0.5, MOON_SHADOW_LUX * 2.0, MOON_SHADOW_LUX));
    assert!(!casts(SHADOW_HEIGHT * 0.5, 350.0, MOON_SHADOW_LUX));
    assert!(casts(0.5, 100.0, 0.0));
    assert!(!casts(0.5, 0.0, 0.0));
}
