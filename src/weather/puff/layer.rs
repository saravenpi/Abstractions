//! The depth of the cloud band: how high a puff stands in it, which the slices of the drawn layer,
//! the whiteout and the fly-through puffs all read, so what is drawn and what the camera flies
//! through are one volume.
//!
//! The band is drawn as [`SLICES`] shells, from its base to its top. A puff of body `b` (the cloud
//! the shell draws seen from overhead, cut at a half) has a flat base at the base of the band and
//! reaches `2 b - 1` of its depth: the edge of a puff is a low cloud, its core towers. A slice
//! draws the columns that reach it. The volume is cloud from the base up to the highest slice a
//! column reaches and nothing above it, so no whiteout and no fly-through puff is ever where no
//! slice is drawn.

use crate::render::smoothstep;

/// Shells the cloud band is drawn with, the lowest at its base and the highest at its top.
pub const SLICES: u32 = 3;

/// Share of the depth of the band over which the volume eases in under the base and out over
/// the highest slice a column reaches: a few tens of metres. Unit: share of the band.
pub const SOFT: f32 = 0.02;

/// Returns the height of slice `k` in the band, from 0 at its base to 1 at its top.
pub fn slice(k: u32) -> f32 {
    k as f32 / (SLICES - 1) as f32
}

/// Returns the height of the highest slice a column of body `body` reaches, from 0 to 1, or
/// nothing when it reaches none: the base slice draws a body of a half and more.
pub fn reached(body: f32) -> Option<f32> {
    let steps = (SLICES - 1) as f32;
    let reach = 2.0 * body - 1.0;
    (reach >= 0.0).then(|| (reach * steps).floor().min(steps) / steps)
}

/// Returns how much of the volume of the band there is at `share` of its depth over a column of
/// body `body`: 1 from its base to the highest slice the column reaches, 0 under the base, over
/// that slice and in a gap, eased over [`SOFT`] at both ends.
pub fn volume(share: f32, body: f32) -> f32 {
    reached(body).map_or(0.0, |top| {
        smoothstep(-SOFT, 0.0, share) * (1.0 - smoothstep(top, top + SOFT, share))
    })
}

/// Returns whether slice `k` draws a column of body `body`: the shader cuts `body - share / 2`
/// at a half. Twin of the cut in `clouds.wesl`.
#[cfg(test)]
pub fn drawn_at(k: u32, body: f32) -> bool {
    body - 0.5 * slice(k) >= 0.5
}
