//! Tuning constants of the terrain meshes.

/// Widest cell in metres whose triangles are toned apart, each by a stable factor from 0.92 to 1.08. The
/// factors average 1, so a toned level and an untoned one show the same mean colour. Coarser cells are
/// wider than the grain the tone stands for, and carry the colour alone.
pub const TONE_REACH: f32 = 4.0;
