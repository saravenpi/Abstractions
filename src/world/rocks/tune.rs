//! Tuning constants of the rocks: when a quarried rock is drawn again.

/// Share of its size a quarried rock must lose before its chunk draws it smaller, unitless: a rebuild
/// of the props of a chunk is not worth a smaller rock than that.
pub const SHRINK_STEP: f32 = 0.05;

/// Mass in kilograms under which what is left of a rock counts as nothing: the rock is gone.
pub const LEAST: f32 = 1.0e-3;
