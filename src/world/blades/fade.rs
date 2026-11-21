//! How the blades thin out and shrink away with the distance from the camera, as `blades.wesl` draws them,
//! and which bands of a tile the distance wants.
//!
//! Every blade carries a rank from 0 to 1. At a distance the share [`kept`] of the blades stands: those ranked
//! under it. A blade whose rank nears that share shrinks to nothing before it goes, and every blade shrinks to
//! nothing before [`END`], so nothing on the ground ever pops in or out.

use std::ops::Range;

use super::tune::{BANDS, END, FADE, FULL, SHORT, SOFT};

/// Every rank: the blades of every band at once.
pub const ALL: Range<f32> = 0.0..1.0;

/// Returns the smooth step from 0 at `low` to 1 at `high` of `x`, as the shader's `smoothstep`.
fn smooth(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the share of the blades, 0 to 1, that stands `distance` metres from the camera on fully covered
/// ground: all within [`FULL`], then falling with the square of the distance.
pub fn kept(distance: f32) -> f32 {
    let r = FULL / distance.max(FULL);
    r * r
}

/// Returns the height of the blade of `rank` that stands `distance` metres from the camera on ground covered
/// `thick`, as a share of its full height: 0 where it does not stand, shrinking as the distance thins it out,
/// as it nears [`END`] and towards the [`SHORT`] far blades.
pub fn grown(rank: f32, distance: f32, thick: f32) -> f32 {
    let keep = kept(distance) * thick;
    if rank >= keep {
        return 0.0;
    }
    let thin = ((1.0 - rank / keep) / SOFT).clamp(0.0, 1.0);
    let end = 1.0 - smooth(END - FADE, END, distance);
    thin * end * (1.0 + (SHORT - 1.0) * smooth(FULL, END, distance))
}

/// Returns the ranks of the blades of `band`: from the top of the band before it to its own top.
pub fn ranks(band: usize) -> Range<f32> {
    let low = band.checked_sub(1).map_or(0.0, |below| BANDS[below]);
    low..BANDS[band]
}

/// Returns the distance in metres from the camera within which `band` holds a blade that stands: where the share
/// kept passes the lowest rank of the band, and [`END`] for the farthest band.
pub fn reach(band: usize) -> f32 {
    let low = ranks(band).start;
    match low > 0.0 {
        true => (FULL / low.sqrt()).min(END),
        false => END,
    }
}
