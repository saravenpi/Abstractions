//! Pure animation curves: easing ramps, wing angles, glide gates, blinking and path curvature.

use std::f32::consts::TAU;

/// Hermite ramp from 0 at `low` to 1 at `high`.
pub fn ramp(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the wing angle in radians above the body plane for a flap phase.
///
/// `glide` from 0 (flapping) to 1 (wings held still at `rest`).
pub fn wing_angle(phase: f32, glide: f32, amplitude: f32, rest: f32) -> f32 {
    rest + (1.0 - glide) * amplitude * phase.sin()
}

/// Returns how much a creature glides at `time`, from 0 to 1, in bursts of `rate` radians per second.
pub fn glide(time: f32, seed: f32, rate: f32) -> f32 {
    ramp(0.3, 0.75, (time * rate + seed * TAU).sin())
}

/// Returns the soft blink brightness from 0 to 1, which only shines at night.
pub fn blink(phase: f32, daylight: f32) -> f32 {
    ramp(0.15, 0.85, 0.5 + 0.5 * phase.sin()) * (1.0 - ramp(0.1, 0.45, daylight))
}

/// Returns the slowly varying turn rate in radians per second of a wandering flock.
pub fn curve(time: f32, seed: f32) -> f32 {
    0.2 * (time * 0.11 + seed).sin() + 0.1 * (time * 0.29 + seed * 2.0).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blinks stay in range and die at noon, wings fold to rest while gliding.
    #[test]
    fn curves_stay_bounded() {
        for step in 0..200 {
            let phase = step as f32 * 0.37;
            assert!((0.0..=1.0).contains(&blink(phase, 0.0)));
            assert_eq!(blink(phase, 1.0), 0.0);
            assert!((0.0..=1.0).contains(&glide(phase, 0.3, 0.5)));
            assert!(curve(phase, 1.0).abs() <= 0.3);
        }
        assert_eq!(wing_angle(1.2, 1.0, 0.8, 0.3), 0.3);
        assert!((wing_angle(1.5707964, 0.0, 0.8, 0.3) - 1.1).abs() < 1e-4);
    }
}
