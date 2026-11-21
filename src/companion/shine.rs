//! The glow: the face colour on the screen, the antenna and the thruster, and the weak light they cast.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::face::Expression;
use super::face::machine::Face;
use super::motion::{Float, blend};
use super::puppet::Parts;
use super::screen::Display;
use super::tune::{GLOW_HDR, LAMP_LUX, LAMP_RADIUS, LIGHT_POWER, SCREEN_HDR, THRUST_GLOW};
use crate::render::{Palette, PlaceLight, reach, scene_lux};

/// How fast the colour follows the expression, per second.
const TINT_RATE: f32 = 6.0;

/// What a companion glows from: its face and brain if it has them, its motion, its parts, its
/// display and its pose, whose scale is its size.
type Glowing = (
    Option<&'static Face>,
    Option<&'static Brain>,
    &'static Float,
    &'static Parts,
    &'static mut Display,
    &'static Transform,
);

/// The light of the place the exposure of the camera is metered from, where there is one.
type Ambient<'w> = (Option<Res<'w, PlaceLight>>, Option<Res<'w, Palette>>);

/// Marks the point light in front of the screen.
#[derive(Component, Debug, Clone, Copy)]
pub struct Lamp;

/// Returns how brightly the screen and the light glow for a face, from 0 to 1.
///
/// The sleepy face is dim only while the companion really dozes. Awake at night it wears the
/// same face and glows like a calm one, so the night light keeps its full strength.
pub fn glow(expr: Expression, dozing: bool) -> f32 {
    match (expr, dozing) {
        (Expression::Sleepy, false) => Expression::Neutral.glow(),
        _ => expr.glow(),
    }
}

/// Returns the share of its power the lamp gives under `lux` of light on the ground of the
/// place: all of it in the dark, none from [`LAMP_LUX`] up, and in between it falls evenly with
/// each doubling of the light.
pub fn dark(lux: f32) -> f32 {
    let span = (LAMP_LUX.1 / LAMP_LUX.0).log2();
    1.0 - ((lux.max(1.0) / LAMP_LUX.0).log2() / span).clamp(0.0, 1.0)
}

/// Returns the light power in lumens under `lux` of light of the place for a companion at
/// `scale` times its size: a lamp that many times smaller has that many times squared less
/// surface, at the same brightness.
pub fn lumens(lux: f32, scale: f32) -> f32 {
    LIGHT_POWER * dark(lux) * scale * scale
}

/// Returns the unlit colour of `tint` at `gain` times white.
fn hdr(tint: Vec3, gain: f32) -> Color {
    Color::linear_rgb(tint.x * gain, tint.y * gain, tint.z * gain)
}

/// Returns the colour one companion glows in, eased toward its face, and how hard its thruster
/// works, eased toward how far it flies. A companion with no face of its own glows calm.
fn want(
    face: Option<&Face>,
    brain: Option<&Brain>,
    float: &Float,
    display: &Display,
    share: f32,
) -> (Vec3, f32) {
    let expr = face.map_or(Expression::Neutral, Face::shown);
    let dozing = brain.is_some_and(|brain| brain.mind.dozing);
    let tint = display.tint.lerp(expr.tint() * glow(expr, dozing), share);
    (
        tint,
        display.thrust + (float.flight - display.thrust) * share,
    )
}

/// Paints the screen and the glowing parts of `display` in `tint`, the glow flared by `thrust`.
fn paint(materials: &mut Assets<StandardMaterial>, display: &Display, (tint, thrust): (Vec3, f32)) {
    let flare = GLOW_HDR * (1.0 + THRUST_GLOW * thrust);
    let paints = [
        (&display.face, hdr(tint, SCREEN_HDR)),
        (&display.glow, hdr(tint, flare)),
    ];
    for (handle, base) in paints {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = base;
            material.emissive = LinearRgba::BLACK;
        }
    }
}

/// Sets the colour and the power of the light in the head of a companion of the size `scale`
/// under `lux` of light of the place, and writes only the values that changed. The range is the
/// reach of the lamp at full power at that size, and the source shrinks with the companion.
fn light(
    heads: &Query<&Children>,
    lamps: &mut Query<&mut PointLight, With<Lamp>>,
    parts: &Parts,
    (tint, lux, scale): (Vec3, f32, f32),
) {
    let next = (hdr(tint, 1.0), lumens(lux, scale));
    let size = (reach(lumens(0.0, scale)), LAMP_RADIUS * scale);
    for &child in heads.get(parts.head).into_iter().flatten() {
        if let Ok(mut lamp) = lamps.get_mut(child) {
            if (lamp.color, lamp.intensity) != next {
                (lamp.color, lamp.intensity) = next;
            }
            if (lamp.range, lamp.radius) != size {
                (lamp.range, lamp.radius) = size;
            }
        }
    }
}

/// Eases the colour of every companion toward its own expression and applies it to its screen,
/// its glowing parts and its light.
///
/// The light has no shadows. It is a lamp for the dark: it follows the light of the place the
/// exposure is metered from, off by day and at full strength at night, and where there is no
/// place, as in space, it is on. In flight the thruster and the antenna glow brighter, so the
/// companion leaves a spark in the air beside the player.
pub fn shine(
    time: Res<Time>,
    (place, palette): Ambient,
    mut bots: Query<Glowing, With<Companion>>,
    (heads, mut lamps): (Query<&Children>, Query<&mut PointLight, With<Lamp>>),
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let share = blend(TINT_RATE, time.delta_secs());
    let lux = match (place, palette) {
        (Some(place), Some(palette)) => scene_lux(&place.0, palette.moon_lux),
        _ => 0.0,
    };
    for (face, brain, float, parts, mut display, at) in &mut bots {
        let (tint, thrust) = want(face, brain, float, &display, share);
        light(&heads, &mut lamps, parts, (tint, lux, at.scale.x));
        let still = (thrust - display.thrust).abs() < 1e-3;
        if still && tint.distance(display.tint) < 1e-4 {
            continue;
        }
        (display.tint, display.thrust) = (tint, thrust);
        paint(&mut materials, &display, (tint, thrust));
    }
}
