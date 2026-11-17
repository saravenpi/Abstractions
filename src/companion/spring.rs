//! The critically damped spring that makes the companion follow softly, never rigidly.

use bevy::prelude::*;

use super::tune::{CATCH_RATE, CATCH_SPAN, FOLLOW_RATE};

/// Hermite ramp from 0 at `low` to 1 at `high`.
pub fn ramp(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Advances a critically damped spring by `dt` seconds and returns the new offset and velocity.
///
/// `offset` is the position minus the target and `rate` the natural frequency in radians per
/// second. The step is the exact solution for a fixed target, so it is stable for any `dt` and a
/// spring released at rest never crosses its target.
pub fn step(offset: Vec3, vel: Vec3, rate: f32, dt: f32) -> (Vec3, Vec3) {
    let decay = (-rate * dt).exp();
    let pull = vel + offset * rate;
    (
        (offset + pull * dt) * decay,
        (vel - pull * rate * dt) * decay,
    )
}

/// Returns the spring rate at `dist` meters from the target: soft nearby, stiff when left behind.
pub fn stiffness(dist: f32) -> f32 {
    FOLLOW_RATE + (CATCH_RATE - FOLLOW_RATE) * ramp(CATCH_SPAN.0, CATCH_SPAN.1, dist)
}

/// Returns the spring rate for a body at `offset` from its target moving at `vel`.
///
/// It is the rate for that distance, raised as far as needed to brake a body that closes in
/// fast, up to the catch-up rate. Without that, a body arriving at speed where the spring is
/// soft would fly past its target.
pub fn rate(offset: Vec3, vel: Vec3) -> f32 {
    let dist = offset.length().max(1e-3);
    let closing = (-vel.dot(offset) / dist).max(0.0);
    stiffness(dist).max((closing / dist).min(CATCH_RATE))
}
