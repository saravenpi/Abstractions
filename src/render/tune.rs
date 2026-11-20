//! Tuning constants of the pixel look, the same on every screen.

/// Rows of game pixels the screen shows, whatever its size, in game pixels. The scale is the
/// whole number of window pixels per game pixel nearest to the window height over this, as a
/// pixel-perfect camera picks the whole scale nearest to its reference resolution
/// (https://docs.unity3d.com/Manual/urp/2d-pixelperfect-ref.html). 360 divides 720, 1080, 1440,
/// 1800 and 2160 exactly.
pub const REFERENCE_ROWS: u32 = 360;

/// Colour levels per channel kept by the banding, in levels.
pub const LEVELS: f32 = 24.0;
