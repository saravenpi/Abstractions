//! What the colour of a triangle reads of the ground under it, and how much turf it holds.

use super::patch::Patch;
use super::tune::{COARSE, FINE, ROCK_ANGLE, ROCK_BAND};
use crate::world::SEA_LEVEL;
use crate::world::lands::Reading;

/// Quarters of turf a triangle can hold, and the lift in metres of the second uv `y` per quarter: the
/// `LEVELS` and `BAND` of `turf.wesl`.
pub const TURF_LEVELS: f32 = 4.0;

/// Lift of the second uv `y` per quarter of turf.
pub const TURF_BAND: f32 = 512.0;

/// Metres above the sea level the bare land of a place is read at least, where the mesh draws it above the sea.
const ABOVE: f32 = 0.01;

/// Returns the slope, rise over run, of the facet over vertices `c` against the local up.
pub fn facet(patch: &Patch, c: [[usize; 2]; 3]) -> f32 {
    let p = c.map(|[i, j]| patch.at(i, j));
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let cos = normal.dot(patch.up(c[0][0], c[0][1]));
    (1.0 - cos * cos).max(0.0).sqrt() / cos.max(0.01)
}

/// Returns what the colour reads of a triangle drawn at the mean height `drawn`: the `reading` of its cell
/// at the fixed scale, with the drawn height where the triangle lies under the sea, so a coast follows the
/// mesh. Without a reading, on the legacy plane, which has no globe to read, the drawn height and the
/// slope `facet` gives.
pub fn read(drawn: f32, reading: Option<Reading>, facet: impl FnOnce() -> f32) -> Reading {
    match reading {
        Some(r) if drawn > SEA_LEVEL => Reading {
            height: r.height.max(SEA_LEVEL + ABOVE),
            ..r
        },
        Some(r) => Reading { height: drawn, ..r },
        None => Reading {
            height: drawn,
            slope: facet(),
        },
    }
}

/// Returns the quarters of turf, 0 to 4, a triangle holds: the mean thickness of its three corners, rounded.
pub fn level(v: &[[f32; 3]; 3]) -> f32 {
    let thick = (v[0][2] + v[1][2] + v[2][2]) / 3.0;
    (thick.clamp(0.0, 1.0) * TURF_LEVELS).round()
}

/// Returns the scale in metres a cell `cell` metres wide is classed at: its own width, from [`FINE`] to
/// [`COARSE`].
pub fn scale(cell: f32) -> f32 {
    cell.clamp(FINE, COARSE)
}

/// Returns the share from 0 to 1 of rock in a coarse cell whose land rises `slope` over [`COARSE`] metres: a
/// smooth step over [`ROCK_BAND`] degrees around [`ROCK_ANGLE`].
pub fn rock_share(slope: f32) -> f32 {
    let degrees = slope.atan().to_degrees();
    let t = ((degrees - (ROCK_ANGLE - ROCK_BAND / 2.0)) / ROCK_BAND).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
