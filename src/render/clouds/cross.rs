//! How deep the camera is inside real cloud: the volume of the drawn slices at its height over the
//! cloud the field holds at that very place, eased so that the edge of a puff is a soft change.

use bevy::prelude::*;

use crate::weather::volume;

/// Seconds over which the cloud around the camera eases in and out.
pub const EASE: f32 = 0.3;

/// How deep the camera is in cloud: 0 in clear air, 1 inside a puff between the base of the band
/// and the highest slice drawn over it.
///
/// The whiteout, the fog, the ship and the companion read it. It stays 0 while clouds are
/// switched off, on a planet without clouds, and in the cloud band wherever the sky is clear.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct CloudCrossing {
    /// Depth in cloud, from 0 to 1.
    pub depth: f32,
}

/// Returns the height of `altitude` in the band from `base` to `top`, from 0 at the base to 1 at
/// the top, or nothing when the sky has no band.
fn band_share(altitude: f32, base: f32, top: f32) -> Option<f32> {
    (top > base).then(|| (altitude - base) / (top - base))
}

/// Returns how much of the cloud volume is at `altitude` in the band from `base` to `top` over a
/// column of body `body`, as the slices of the layer draw it: see [`volume`]. 0 outside the band,
/// over the highest slice the column reaches and in a gap.
pub fn depth_at(altitude: f32, (base, top): (f32, f32), body: f32) -> f32 {
    band_share(altitude, base, top).map_or(0.0, |share| volume(share, body))
}

/// Returns the cloud around the camera `seconds` after it was `around`, with `density` of
/// cloud at its place now: it eases toward `density` over about [`EASE`] seconds while the
/// camera is in the cloud `band`, and out of the band it follows at once, unseen.
pub fn eased(around: f32, density: f32, band: f32, seconds: f32) -> f32 {
    let kept = match band > 0.0 {
        true => (-seconds.max(0.0) / EASE).exp(),
        false => 0.0,
    };
    density + (around - density) * kept
}
