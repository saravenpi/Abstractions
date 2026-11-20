//! Tuning of the bake of the galaxy light.

/// Seconds the last bake must be old before the next one starts, so fast flight through many
/// galaxies bakes a few times, not back to back. Unit: seconds of the app clock.
pub const MIN_GAP: f64 = 4.0;

/// The bake runs on the cores of the machine divided by this: half of them leave the rest to
/// the game. Unit: divisor.
pub const SHARE: usize = 2;

/// Seconds a new map takes to fade in over the one on show, so the sky grows between two bakes
/// and does not change in steps: as long as the minimum gap between bakes. Unit: seconds of the
/// app clock.
pub const FADE: f64 = MIN_GAP;
