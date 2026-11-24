//! Tuning constants of the pixel look, consistent across every window size.

/// Maximum rows of game pixels the screen shows: fullscreen and any taller window stay at this
/// vertical detail instead of getting blockier than smaller windows, while smaller windows never
/// exceed their physical height.
pub const REFERENCE_ROWS: u32 = 540;

/// Colour levels per channel kept by the banding, in levels.
pub const LEVELS: f32 = 40.0;
