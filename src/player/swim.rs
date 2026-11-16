//! Buoyancy: floating at the sea surface and swimming up.

use super::EYE_OFFSET;
use crate::world::SEA_LEVEL;

/// Eye height relative to the sea surface while floating.
const FLOAT_EYE: f32 = -0.15;

/// Strength of the spring pulling the body to its floating height.
const SPRING: f32 = 6.0;

/// Largest vertical speed of the buoyancy spring.
const MAX_SPRING_SPEED: f32 = 3.0;

/// Upward speed while the jump key is held in water.
const SWIM_UP_SPEED: f32 = 4.0;

/// Returns whether a body with this center height is in deep enough water to swim.
pub(super) fn is_swimming(center_y: f32) -> bool {
    center_y + EYE_OFFSET < SEA_LEVEL
}

/// Returns the vertical velocity that floats the body at the surface, or swims up when asked.
pub(super) fn buoyancy_velocity(center_y: f32, swim_up: bool) -> f32 {
    let target = SEA_LEVEL + FLOAT_EYE - EYE_OFFSET;
    let spring = ((target - center_y) * SPRING).clamp(-MAX_SPRING_SPEED, MAX_SPRING_SPEED);
    if swim_up {
        spring.max(SWIM_UP_SPEED)
    } else {
        spring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deep bodies rise and a floating body is roughly still.
    #[test]
    fn buoyancy_pulls_toward_surface() {
        assert!(buoyancy_velocity(SEA_LEVEL - 3.0, false) > 0.0);
        let floating = SEA_LEVEL + FLOAT_EYE - EYE_OFFSET;
        assert!(buoyancy_velocity(floating, false).abs() < 1e-4);
        assert!(buoyancy_velocity(floating, true) >= SWIM_UP_SPEED);
    }

    /// Only bodies whose eyes are under water swim.
    #[test]
    fn swimming_depends_on_eye_depth() {
        assert!(is_swimming(SEA_LEVEL - EYE_OFFSET - 0.1));
        assert!(!is_swimming(SEA_LEVEL + 1.0));
    }
}
