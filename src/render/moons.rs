//! Moons and rings as the sky sees them: the phase a body shows, the shadows the bodies of one
//! planet cast on each other and on the camera, and the light a real moon gives at night.
//!
//! `bodies.wesl` and `rings.wesl` draw the same ring coverage and shadows; keep them in step.

use std::f32::consts::TAU;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::atmosphere::shadow;
use super::celestial::smoothstep;
use super::nearby::Nearby;
use super::vantage::Vantage;
use crate::universe::{BodyId, PlanetSpec, Rings};

/// Angular radius in radians of the moon the sky once painted: a real body seen this large and
/// full gives [`super::Palette::MOON_LUX`].
const MOON_ANGLE: f64 = 0.045;

/// Most a body lights the night against that full moon, however much of the sky it fills.
const GIANT_GAIN: f64 = 3.0;

/// Ringlets across the width of the rings.
const RINGLETS: f32 = 5.0;

/// Share of the coverage lost in the gaps between ringlets.
const RING_DEPTH: f32 = 0.6;

/// Width of the soft inner and outer edges of the rings, as a share of their outer radius.
const RING_EDGE: f32 = 0.04;

/// A real body that lights the night: where it stands and how much light it gives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glow {
    /// Unit vector from the camera toward the body, in render axes.
    pub dir: Vec3,
    /// Sine of its height above the horizon.
    pub height: f32,
    /// Its light against a full moon of [`MOON_ANGLE`] high in the sky: its phase, its size, the
    /// shadows on it and how high it stands, together.
    pub share: f32,
}

/// Returns the lit share of the disc of a body `body` metres from the star, seen from `eye`:
/// 1 when it is full, 0 when it is new.
pub fn phase(body: DVec3, eye: DVec3) -> f32 {
    let to_star = (-body).normalize_or(DVec3::X);
    let to_eye = (eye - body).normalize_or(to_star);
    (0.5 + 0.5 * to_star.dot(to_eye)) as f32
}

/// Returns the share of the light the rings stop `radius` metres from the centre of their
/// planet: their density, softened at both edges and broken into ringlets.
pub fn coverage(rings: &Rings, radius: f64) -> f32 {
    let u = (radius / rings.outer) as f32;
    let inner = (rings.inner / rings.outer) as f32;
    if !(inner..=1.0).contains(&u) {
        return 0.0;
    }
    let t = (u - inner) / (1.0 - inner).max(1.0e-4);
    let edge =
        smoothstep(inner, inner + RING_EDGE, u) * (1.0 - smoothstep(1.0 - RING_EDGE, 1.0, u));
    let wave = TAU * RINGLETS * t + 1.7 * (TAU * 1.3 * t).sin();
    rings.density * edge * (1.0 - RING_DEPTH * (0.5 + 0.5 * wave.cos()))
}

/// Returns the share of starlight that reaches a point `from` metres from the centre of a
/// ringed planet past its rings, the star in the unit direction `sun`. The rings lie in the
/// plane of the equator, and every planet spins about +Y of the system.
pub fn ring_shadow(from: DVec3, sun: DVec3, rings: &Rings) -> f32 {
    if from.y * sun.y >= 0.0 {
        return 1.0;
    }
    let hit = from - sun * (from.y / sun.y);
    1.0 - coverage(rings, hit.length())
}

/// Returns the share of starlight that reaches the point `at`, metres from the star, past the
/// bodies of `family` at `time`: every one but `own` by its globe, and every one by its rings.
pub fn eclipse<'a>(
    at: DVec3,
    family: impl Iterator<Item = &'a PlanetSpec>,
    own: BodyId,
    time: f64,
) -> f32 {
    let sun = (-at).normalize_or(DVec3::X);
    family
        .map(|body| {
            let from = at - body.position(time);
            let globe = match body.id == own {
                true => 1.0,
                false => shadow(from, sun, body.radius),
            };
            globe
                * body
                    .rings
                    .as_ref()
                    .map_or(1.0, |r| ring_shadow(from, sun, r))
        })
        .product()
}

/// Returns how a body lights the night of the camera at `vantage`.
fn glow(vantage: &Vantage, near: &Nearby, body: &PlanetSpec, time: f64) -> Glow {
    let observer = &vantage.observer;
    let pos = body.position(time);
    let offset = pos - observer.eye;
    let dir = offset.normalize_or(DVec3::Y);
    let angle = body.radius / offset.length().max(body.radius);
    let size = (angle / MOON_ANGLE).powi(2).min(GIANT_GAIN) as f32;
    let height = dir.dot(observer.up()) as f32;
    let shaded = near.eclipse(pos, body.id, time);
    let risen = smoothstep(-0.02, 0.1, height);
    Glow {
        dir: (observer.to_render * dir).as_vec3(),
        height,
        share: phase(pos, observer.eye) * size * shaded * risen,
    }
}

/// Returns the body that lights the night of the camera at `vantage` most: the brightest of
/// the planet and moons around it but the one it stands on. A planet without moons has none.
pub fn moonlight(vantage: &Vantage, near: &Nearby, time: f64) -> Option<Glow> {
    let index = u8::try_from(vantage.body).ok()?;
    near.family(index)
        .filter(|body| !vantage.is(body.id))
        .map(|body| glow(vantage, near, body, time))
        .max_by(|a, b| a.share.total_cmp(&b.share))
}
