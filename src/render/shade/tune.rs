//! Tuning constants of the shading by distance and height. The shader constants of the same
//! names in `daylight.wesl` hold the same numbers, and a test keeps them equal.

/// Share of the last shadow cascade, counted back from its far end, over which the ground shader
/// fades the shadows out, so they end in a fade and not at a line. A share of 1.
#[cfg(test)]
pub const SHADOW_FADE: f32 = 0.2;

/// Distances from the camera in metres between which the ground shader fades ambient occlusion
/// out: full up to the first, gone from the second. The effect is made of what is near, and it
/// halos at the seams of far chunks.
#[cfg(test)]
pub const OCCLUSION_RANGE: (f32, f32) = (30.0, 90.0);

/// Height of the camera above the ground in metres from which the ambient occlusion pass is
/// taken off the camera: the ground it could darken is all beyond [`OCCLUSION_RANGE`].
pub const OCCLUSION_CEILING: f32 = 90.0;

/// Metres above [`OCCLUSION_CEILING`] the camera must climb before the pass goes, so it does not
/// flicker on and off at the ceiling.
pub const OCCLUSION_MARGIN: f32 = 10.0;
