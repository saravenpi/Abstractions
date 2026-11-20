//! The heart: how fast it beats and how an artery pulses with it.

use std::f32::consts::TAU;

use super::tune::HEART;

/// Returns the heart rate of a body of `mass` kilograms, in beats per second.
pub fn heart_rate(mass: f32) -> f32 {
    HEART * mass.max(1e-3).powf(-0.25)
}

/// Returns the flow of an artery at time `t` seconds, relative to its mean, for a heart that beats
/// `rate` times a second: the systolic half of each beat carries it all, with a mean of 1.
pub fn pulse(t: f32, rate: f32) -> f32 {
    let wave = (TAU * rate * t).sin().max(0.0);
    4.0 * wave * wave
}
