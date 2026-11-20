//! Tuning constants of the scene lights.

/// Lowest height of a light above the horizon, as a sine, at which its shadows are cast.
pub const SHADOW_HEIGHT: f32 = 0.02;

/// Distance from the camera in metres where the first shadow cascade ends.
pub const FIRST_CASCADE: f32 = 28.0;

/// Distance from the camera in metres where the last shadow cascade ends. Beyond it the shadow
/// maps give no shadow, and the ground shader fades the last fifth of the cascade out.
pub const SHADOW_RANGE: f32 = 110.0;

/// Illuminance in lux of the moon below which it casts no shadow: a sixth of a full moon, about
/// what the night ambient light already gives, so a shadow would not read.
pub const MOON_SHADOW_LUX: f32 = 60.0;
