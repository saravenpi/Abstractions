//! The wave of a shaking view: a small offset and tilt that never repeat and never jolt.

use bevy::prelude::*;

/// Largest offset of the view in metres and largest tilt in radians, at full shake.
pub const REACH: (f32, f32) = (0.012, 0.004);

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
