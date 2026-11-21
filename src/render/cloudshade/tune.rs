//! Tuning constants of the cloud shadows. The shader constants `SHADE_AT`, `SHADE_DEPTH` and
//! `SHADE_SUN` in `daylight.wesl` hold the same numbers, and a test keeps them equal.

/// Optical depth of a full puff for the direct beam. The beam that gets through is
/// `exp(-depth)`: 0.135 for 2, which is what a thin cloud deck leaves of the sun. A share of 1
/// stands for no light. Unit: none.
pub const OPTICAL_DEPTH: f32 = 2.0;

/// Sines of the height of the sun between which the shadows come on: none below the first,
/// where the beam crosses many clouds and the air takes the light anyway, all above the second.
pub const SUN_EDGE: (f32, f32) = (0.03, 0.15);

/// Texels of the light block the shadows take: the planet axes in three, the shell, and the turn
/// of the puffs.
pub const HEAD: usize = 5;
