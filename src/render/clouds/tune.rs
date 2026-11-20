//! Tuning of the cover maps and of the shell mesh of the clouds.

use crate::weather::{SNAPSHOT, TEXELS};

/// Texels of the next snapshot worked out per second of frame time, on top of [`STAGE_FLOOR`]:
/// twice the 24,576 texels per second that fill one snapshot of 393,216 texels within its
/// [`SNAPSHOT`] seconds of 16, so one map is always done in half its period, and so are two
/// maps that share the budget. Unit: texels per second.
pub const STAGE_RATE: f32 = 2.0 * TEXELS as f32 / SNAPSHOT as f32;

/// Fewest texels of the next snapshot worked out in one frame, shared by every map, so a very
/// fast frame still moves the work on. Unit: texels.
pub const STAGE_FLOOR: usize = 256;

/// Returns the texels of the next snapshot to work out in a frame of `seconds`, for all the
/// maps together: [`STAGE_FLOOR`] and [`STAGE_RATE`] over the frame time, which is capped at one
/// second.
pub fn budget(seconds: f32) -> usize {
    STAGE_FLOOR + (seconds.clamp(0.0, 1.0) * STAGE_RATE) as usize
}

/// Share of the distance from the camera to the nearest point of the shell that the mesh may
/// lie off the true sphere. The vertices carry the true normals, so only the outline can show
/// a facet, and its sag stays a hundredth of how far away it is. Unit: share of a distance.
pub const SEEN_ERROR: f32 = 0.01;

/// How many times finer than needed a new mesh is built, so the camera can move closer before
/// it has to be built again. Unit: factor.
pub const SPARE: f32 = 2.0;

/// How many times finer than needed the mesh may get before it is built again, so a camera
/// that climbs away does not keep a mesh much finer than it needs. Unit: factor.
pub const COARSEN: f32 = 4.0;
