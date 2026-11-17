//! The shake of the view in rough air: small, smooth and never a jolt.

use bevy::prelude::*;

use crate::render::smoothstep;

/// Largest offset of the view in metres and largest tilt in radians, at full shake.
pub const REACH: (f32, f32) = (0.012, 0.004);

/// Dynamic pressure in pascals at which the hull starts to buffet and at which it buffets fully.
const BUFFET: (f32, f32) = (4.0e3, 6.0e4);

/// Share of the full shake in the middle of the cloud layer.
const CLOUD_SHARE: f32 = 0.6;

/// Rate per second at which the shake follows the air.
pub const FOLLOW: f32 = 3.0;

/// How hard the view shakes, from 0 to 1, eased so that it never steps.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Buffet {
    /// Strength of the shake.
    pub level: f32,
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

/// Returns the offset and the tilt of the view at `time` seconds for a shake of `level`.
///
/// Each axis is the sum of two slow sines whose rates share no period, so the motion never
/// repeats visibly and has no step: its speed is bounded by the rates.
pub fn shake(time: f32, level: f32) -> (Vec3, Quat) {
    let wave = |a: f32, b: f32, phase: f32| {
        0.6 * (time * a + phase).sin() + 0.4 * (time * b + 2.0 * phase).sin()
    };
    let offset = Vec3::new(wave(23.0, 37.1, 0.0), wave(29.3, 41.7, 1.3), 0.0);
    let tilt = Vec3::new(
        wave(19.1, 31.3, 2.1),
        wave(17.3, 27.7, 0.7),
        wave(21.7, 33.1, 4.0),
    );
    let turn = tilt * (REACH.1 * level);
    (
        offset * (REACH.0 * level),
        Quat::from_euler(EulerRot::XYZ, turn.x, turn.y, turn.z),
    )
}
