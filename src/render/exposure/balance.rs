//! The white balance of the game camera: a von Kries adaptation of the whole picture toward the
//! colour of the light that reaches it.
//!
//! An eye and a camera adapt to the illuminant: under a red star the hull is still read as pale
//! and the robot keeps its colours. The adaptation is the CIE CAT02 one, partial by the degree
//! `D = F (1 - exp(-(L_A + 42) / 92) / 3.6)` with `F` 1 for an average surround and `L_A` the
//! luminance of the adapting field in cd/m2
//! (https://en.wikipedia.org/wiki/CIECAM02). The degree is near 1 for a bright scene and falls
//! to 0.82 in the dark. It is slow, as the eye is: about 90 percent in a minute
//! (Fairchild and Reniff, 1995, https://digitalarchive.rit.edu/xmlui/handle/1850/3112).
//! Bevy's colour grading applies the transform in LMS space to the whole image after the exposure,
//! so the sky, the fog and the unlit materials adapt with the lit ones.

use std::f32::consts::PI;

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::view::ColorGrading;

use super::scene_lux;
use super::tune::{ADAPT_SECONDS, MEAN_ALBEDO, SUN_LIGHT, SURROUND, WHITE_STEP};
use crate::render::GameCamera;
use crate::render::daylight::PlaceLight;
use crate::render::palette::Palette;
use crate::settings::{Persistent, Settings};

/// The CIE 1931 chromaticity of the D65 white point, the one Bevy's colour grading maps to white.
pub const D65: Vec2 = Vec2::new(0.31272, 0.32903);

/// The white the camera has adapted to, as a CIE 1931 xy chromaticity, and whether it was set
/// from the light yet.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Balanced {
    /// Chromaticity of the white the picture is balanced to.
    pub white: Vec2,
    /// Whether the white was ever set from the light of a place.
    pub set: bool,
}

impl Default for Balanced {
    fn default() -> Self {
        Self {
            white: D65,
            set: false,
        }
    }
}

/// Returns the CIE 1931 xy chromaticity of the linear sRGB colour `rgb`, or D65 for black.
pub fn chromaticity(rgb: Vec3) -> Vec2 {
    let x = 0.412_456_4 * rgb.x + 0.357_576_1 * rgb.y + 0.180_437_5 * rgb.z;
    let y = 0.212_672_9 * rgb.x + 0.715_152_2 * rgb.y + 0.072_175 * rgb.z;
    let z = 0.019_333_9 * rgb.x + 0.119_192 * rgb.y + 0.950_304_1 * rgb.z;
    let sum = x + y + z;
    match sum > 1.0e-9 {
        true => Vec2::new(x / sum, y / sum),
        false => D65,
    }
}

/// Returns the CAT02 degree of adaptation, 0 to 1, to a scene lit by `lux` on level ground: the
/// adapting field is a mean grey ground, so `L_A` is [`MEAN_ALBEDO`] times the lux over pi.
pub fn degree(lux: f32) -> f32 {
    let field = MEAN_ALBEDO * lux.max(0.0) / PI;
    SURROUND * (1.0 - (-(field + 42.0) / 92.0).exp() / 3.6)
}

/// Returns the white an eye adapts to under `light`, the linear colour of the light that reaches
/// the camera, lighting a scene of `lux`: D65 moved toward the light, measured against the light
/// of the Sun ([`SUN_LIGHT`], which the game takes for neutral), by [`degree`].
pub fn adapted(light: Vec3, lux: f32) -> Vec2 {
    D65.lerp(chromaticity(light / SUN_LIGHT), degree(lux))
}

/// Returns the temperature and tint of Bevy's colour grading that map the white `white` to
/// white: Bevy puts its white point at D65 plus (-temperature, tint).
pub fn grading(white: Vec2) -> (f32, f32) {
    (D65.x - white.x, white.y - D65.y)
}

/// Returns `white` moved toward `target` over `dt` seconds, with a time constant of
/// [`ADAPT_SECONDS`].
pub fn eased(white: Vec2, target: Vec2, dt: f32) -> Vec2 {
    white.lerp(target, 1.0 - (-dt.max(0.0) / ADAPT_SECONDS).exp())
}

/// Writes the white balance `white` to the colour grading of the camera, and adds the grading
/// when the camera has none. A change under [`WHITE_STEP`] is not written.
fn apply(
    white: Vec2,
    cameras: &mut Query<(Entity, Option<&mut ColorGrading>), With<GameCamera>>,
    commands: &mut Commands,
) {
    let (temperature, tint) = grading(white);
    for (camera, grade) in cameras {
        match grade {
            Some(mut grade)
                if (grade.global.temperature - temperature)
                    .abs()
                    .max((grade.global.tint - tint).abs())
                    > WHITE_STEP =>
            {
                grade.global.temperature = temperature;
                grade.global.tint = tint;
            }
            None => {
                let mut grade = ColorGrading::default();
                grade.global.temperature = temperature;
                grade.global.tint = tint;
                commands.entity(camera).insert(grade);
            }
            _ => {}
        }
    }
}

/// Adapts the white of the camera to the colour of the light and writes it as the colour grading
/// white balance, slowly. Without a place, or with the setting off, the white is D65 at once.
pub(super) fn run(
    (time, place, palette): (Res<Time>, Res<PlaceLight>, Res<Palette>),
    settings: Res<Persistent<Settings>>,
    mut balanced: ResMut<Balanced>,
    mut cameras: Query<(Entity, Option<&mut ColorGrading>), With<GameCamera>>,
    mut commands: Commands,
) {
    let live = place.0.air.z > 0.0 && settings.graphics.lighting.white_balance;
    let lux = scene_lux(&place.0, palette.moon_lux);
    let light = place.0.eye_sun.truncate() + PI * place.0.eye_sky.truncate();
    let target = match live {
        true => adapted(light, lux),
        false => D65,
    };
    let white = match balanced.set && live {
        true => eased(balanced.white, target, time.delta_secs()),
        false => target,
    };
    let set = live || balanced.set;
    balanced.set_if_neq(Balanced { white, set });
    apply(balanced.white, &mut cameras, &mut commands);
}
