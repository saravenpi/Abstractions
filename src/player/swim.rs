//! Swim state with hysteresis, the floating height with its bob, and wading speed.

use super::{BODY_LENGTH, BODY_RADIUS, EYE_OFFSET};

/// Eye height above the sea surface while floating.
const FLOAT_EYE: f32 = 0.3;

/// Amplitude of the floating bob in meters.
const BOB_AMPLITUDE: f32 = 0.03;

/// Angular speed of the floating bob in radians per second.
const BOB_RATE: f32 = 2.0;

/// Body center height above the sea below which a walking body starts to swim.
const ENTER_CENTER: f32 = -0.55;

/// Body center height above the sea above which a swimming body near the floor walks again.
const EXIT_CENTER: f32 = -0.5;

/// Largest gap to the floor at which a swimming body switches back to walking.
const EXIT_GAP: f32 = 0.08;

/// Body center height above the sea at which a swimming body leaves the water into the air.
const LEAVE_CENTER: f32 = 0.2;

/// Feet depth in meters at which wading reaches its slowest speed.
const WADE_DEPTH: f32 = 1.2;

/// Slowest walking speed multiplier when wading.
const WADE_MIN: f32 = 0.65;

/// Returns the body center height above the sea at which the body floats, with the bob at a given time.
pub(super) fn float_center(time: f32) -> f32 {
    FLOAT_EYE - EYE_OFFSET + BOB_AMPLITUDE * (time * BOB_RATE).sin()
}

/// Returns whether the body swims next frame, with hysteresis around every threshold.
///
/// `height` is the altitude of the body center above the sea.
pub(super) fn next_swimming(swimming: bool, height: f32, floor_gap: Option<f32>) -> bool {
    if !swimming {
        return height < ENTER_CENTER;
    }
    let on_floor = floor_gap.is_some_and(|gap| gap <= EXIT_GAP) && height > EXIT_CENTER;
    !on_floor && height <= LEAVE_CENTER
}

/// Returns the walking speed multiplier for a body in shallow water, from the altitude of its center.
pub(super) fn wade_factor(height: f32) -> f32 {
    let feet_depth = BODY_LENGTH * 0.5 + BODY_RADIUS - height;
    1.0 - (1.0 - WADE_MIN) * (feet_depth / WADE_DEPTH).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A body that stops swimming at the float height never starts again, and the reverse.
    #[test]
    fn state_does_not_flicker_at_the_shore() {
        for step in 0..60 {
            let time = step as f32 * 0.1;
            let swim_center = float_center(time);
            for tenth in 0..30 {
                let gap = tenth as f32 * 0.01;
                if !next_swimming(true, swim_center, Some(gap)) {
                    assert!(!next_swimming(false, swim_center - gap, None));
                }
            }
        }
        assert!(next_swimming(false, -1.0, None));
        assert!(next_swimming(true, float_center(0.0), None));
        assert!(!next_swimming(true, 0.5, None));
    }

    /// Wading slows the walk with depth and never below the minimum.
    #[test]
    fn wading_slows_with_depth() {
        assert_eq!(wade_factor(3.0), 1.0);
        assert!(wade_factor(-0.3) < wade_factor(0.5));
        assert!((wade_factor(-5.0) - WADE_MIN).abs() < 1e-5);
    }
}
