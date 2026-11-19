//! How deep the camera is inside real cloud: its height in the cloud band times the cloud the
//! field holds at that very place, eased so that the edge of a puff is a soft change.

use bevy::prelude::*;

use crate::render::celestial::smoothstep;

/// Seconds over which the cloud around the camera eases in and out.
pub const EASE: f32 = 0.3;

/// How deep the camera is in cloud: 0 in clear air, 1 inside a puff in the middle of the band.
///
/// The whiteout, the fog, the ship and the companion read it. It stays 0 while clouds are
/// switched off, on a planet without clouds, and in the cloud band wherever the sky is clear.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct CloudCrossing {
    /// Depth in cloud, from 0 to 1.
    pub depth: f32,
}

/// Returns how deep `altitude` is in the band from `base` to `top`: 0 below the base and above
/// the top, 1 in the middle, smooth in between.
pub fn crossing_depth(altitude: f32, base: f32, top: f32) -> f32 {
    if top <= base {
        return 0.0;
    }
    let along = (altitude - base) / (top - base);
    smoothstep(0.0, 1.0, 1.0 - (2.0 * along - 1.0).abs())
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
