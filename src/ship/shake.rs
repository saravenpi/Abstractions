//! The shake of the view: in rough air small, smooth and never a jolt, and after a crash a
//! short hard jolt that dies out in half a second.

use bevy::prelude::*;

pub use crate::speed::shake::{REACH, shake};

use crate::render::smoothstep;

/// Dynamic pressure in pascals at which the hull starts to buffet and at which it buffets fully.
const BUFFET: (f32, f32) = (4.0e3, 6.0e4);

/// Share of the full shake in the middle of the cloud layer.
const CLOUD_SHARE: f32 = 0.6;

/// Rate per second at which the shake follows the air.
pub const FOLLOW: f32 = 3.0;

/// How many times the full shake of rough air a crash shakes the view, and the seconds the
/// jolt takes to die out.
pub const JOLT: (f32, f32) = (5.0, 0.5);

/// Crash speed in metres per second that gives the full jolt; slower crashes give less.
const JOLT_SPEED: f64 = 50.0;

/// How hard the view shakes.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Buffet {
    /// Strength of the shake of rough air, from 0 to 1, eased so that it never steps.
    pub level: f32,
    /// What is left of the jolt of the last crash, from 0 to 1.
    pub jolt: f32,
}

/// Returns the jolt after `dt` seconds: what was left of `jolt`, dying out over the time of
/// [`JOLT`], or the jolt of a crash at `speed` metres per second if one happened.
pub fn jolted(jolt: f32, speed: Option<f64>, dt: f32) -> f32 {
    let left = (jolt - dt / JOLT.1).max(0.0);
    speed.map_or(left, |speed| {
        left.max((speed / JOLT_SPEED).clamp(0.5, 1.0) as f32)
    })
}

/// Returns how hard the air shakes the ship, from 0 to 1, for a depth in the cloud layer from
/// 0 to 1 and a dynamic pressure in pascals.
pub fn roughness(cloud_depth: f32, pressure: f32) -> f32 {
    let buffet = smoothstep(BUFFET.0, BUFFET.1, pressure);
    (CLOUD_SHARE * cloud_depth.clamp(0.0, 1.0) + buffet).min(1.0)
}

/// Returns the level after `dt` seconds of easing from `level` toward `wanted` at `rate` per
/// second, exactly zero once it has faded.
pub fn eased(level: f32, wanted: f32, rate: f32, dt: f32) -> f32 {
    let next = level + (wanted - level) * (1.0 - (-rate * dt).exp());
    if wanted == 0.0 && next < 1e-3 {
        0.0
    } else {
        next
    }
}
