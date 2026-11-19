//! The glow: the face colour on the screen, the antenna and the thruster, and the weak light they cast.

use bevy::prelude::*;

use super::Companion;
use super::brain::Brain;
use super::face::Expression;
use super::face::machine::Face;
use super::motion::{Float, blend};
use super::puppet::Parts;
use super::screen::Display;
use super::sense::CompanionSense;
use super::tune::{GLOW_HDR, LIGHT_DAY_SHARE, LIGHT_POWER, SCREEN_HDR, THRUST_GLOW};

/// How fast the colour follows the expression, per second.
const TINT_RATE: f32 = 6.0;

/// What a companion glows from: its face and brain if it has them, its motion, its parts and its display.
type Glowing = (
    Option<&'static Face>,
    Option<&'static Brain>,
    &'static Float,
    &'static Parts,
    &'static mut Display,
);

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

/// Returns the light power in lumens for a night level from 0 (day) to 1 (night).
pub fn lumens(night: f32) -> f32 {
    LIGHT_POWER * (LIGHT_DAY_SHARE + (1.0 - LIGHT_DAY_SHARE) * night.clamp(0.0, 1.0))
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

/// Eases the colour of every companion toward its own expression and applies it to its screen,
/// its glowing parts and its light.
///
/// The light has no shadows, weakens by day and is at full strength at night. In flight the
/// thruster and the antenna glow brighter, so the companion leaves a spark in the air beside
/// the player.
pub fn shine(
    time: Res<Time>,
    sense: Res<CompanionSense>,
    mut bots: Query<Glowing, With<Companion>>,
    (heads, mut lamps): (Query<&Children>, Query<&mut PointLight, With<Lamp>>),
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let share = blend(TINT_RATE, time.delta_secs());
    for (face, brain, float, parts, mut display) in &mut bots {
        let (tint, thrust) = want(face, brain, float, &display, share);
        for &child in heads.get(parts.head).into_iter().flatten() {
            if let Ok(mut lamp) = lamps.get_mut(child) {
                lamp.color = hdr(tint, 1.0);
                lamp.intensity = lumens(sense.night);
            }
        }
        let still = (thrust - display.thrust).abs() < 1e-3;
        if still && tint.distance(display.tint) < 1e-4 {
            continue;
        }
        (display.tint, display.thrust) = (tint, thrust);
        paint(&mut materials, &display, (tint, thrust));
    }
}
