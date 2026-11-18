//! Slow seeded noise in time: one value per knot, eased between knots, so the weather drifts
//! over minutes and comes back the same for the same seed and second.

use crate::audio::Rand;

/// Seconds between two knots of the broad swell and of the faster ripple on top of it.
const PERIODS: [f64; 2] = [300.0, 97.0];

/// Weight of the faster ripple against the broad swell.
const RIPPLE: f32 = 0.35;

/// Returns the value of knot `index` of channel `channel` on the planet of `seed`, from 0 to 1.
pub fn knot(seed: u64, channel: u64, index: i64) -> f32 {
    Rand::new(
        seed ^ channel.wrapping_mul(0x9E37_79B9_7F4A_7C15),
        index as u64,
    )
    .unit()
}

/// Returns one octave of channel `channel` at `seconds`: knots `period` seconds apart, eased.
fn octave(seed: u64, channel: u64, seconds: f64, period: f64) -> f32 {
    let at = seconds / period;
    let index = at.floor();
    let t = (at - index) as f32;
    let ease = t * t * (3.0 - 2.0 * t);
    let (a, b) = (
        knot(seed, channel, index as i64),
        knot(seed, channel, index as i64 + 1),
    );
    a + (b - a) * ease
}

/// Returns the slow noise of channel `channel` on the planet of `seed` at `seconds` of universe
/// time, from 0 to 1.
pub fn slow(seed: u64, channel: u64, seconds: f64) -> f32 {
    let swell = octave(seed, channel, seconds, PERIODS[0]);
    let ripple = octave(seed, channel + 0x100, seconds, PERIODS[1]);
    (swell + RIPPLE * ripple) / (1.0 + RIPPLE)
}
