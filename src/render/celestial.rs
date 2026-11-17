//! Pure helpers for the sun and moon path and the daylight factor.

use std::f32::consts::TAU;

use bevy::prelude::Vec3;

/// Tilt of the orbit axis in radians, so shadows lean instead of sweeping straight overhead.
const AXIS_TILT: f32 = 0.5;

/// Hermite interpolation of `x` between `a` and `b`, clamped to 0..1.
pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the unit vector pointing at the sun: 0.0 is sunrise in the east, 0.25 is noon.
pub fn sun_direction(phase: f32) -> Vec3 {
    let angle = phase * TAU;
    Vec3::new(
        angle.cos(),
        angle.sin() * AXIS_TILT.cos(),
        angle.sin() * AXIS_TILT.sin(),
    )
}

/// Returns the unit vector pointing at the moon, always opposite the sun.
pub fn moon_direction(phase: f32) -> Vec3 {
    -sun_direction(phase)
}

/// Returns 0 at night and 1 in full day, with a smooth ramp across dawn and dusk.
pub fn daylight(phase: f32) -> f32 {
    smoothstep(-0.12, 0.25, sun_direction(phase).y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sun is up at noon, the moon at midnight, and they are always opposite.
    #[test]
    fn sun_and_moon_swap_places() {
        assert!(sun_direction(0.25).y > 0.8);
        assert!(moon_direction(0.75).y > 0.8);
        assert!(sun_direction(0.75).y < -0.8);
        assert!((sun_direction(0.1) + moon_direction(0.1)).length() < 1e-6);
    }

    /// Daylight is full at noon and zero at midnight.
    #[test]
    fn daylight_follows_the_sun() {
        assert!(daylight(0.25) > 0.99);
        assert!(daylight(0.75) < 0.01);
    }
}
