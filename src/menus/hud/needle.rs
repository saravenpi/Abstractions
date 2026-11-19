//! Where a mark of the compass sits: the angle to its target, its place along the bar, how
//! much it has faded toward the ends, and the row it takes beside marks it would cover.

use bevy::prelude::*;

/// Width of the bar in pixels.
pub const WIDTH: f32 = 360.0;

/// Angle in radians from the middle of the bar to either end.
pub const SPAN: f32 = 1.2;

/// Least gap in pixels between two marks on one row, so their labels stay readable.
pub const APART: f32 = 72.0;

/// Most rows the marks stack in; a mark that finds no free row goes on the last one.
pub const ROWS: usize = 5;

/// Share of the span, at each end, over which a mark fades out.
const FADE: f32 = 0.3;

/// Returns the angle in radians from the view to a target `offset` away, around the local
/// `up`: positive to the right, zero straight ahead, a half turn behind. Nothing for a target
/// straight above or below, or a view that looks straight up or down.
pub fn bearing(offset: Vec3, forward: Vec3, up: Vec3) -> Option<f32> {
    let flat = |v: Vec3| (v - up * v.dot(up)).try_normalize();
    let (to, ahead) = (flat(offset)?, flat(forward)?);
    Some(to.dot(ahead.cross(up)).atan2(to.dot(ahead)))
}

/// Returns where a target at `bearing` sits on the bar, from 0 at its left end to 1 at its
/// right end. A target outside the span waits at the end of its side.
pub fn slot(bearing: f32) -> f32 {
    0.5 + 0.5 * (bearing / SPAN).clamp(-1.0, 1.0)
}

/// Returns how opaque a mark at `bearing` is: fully inside the span, fading to nothing over
/// the last share of it toward each end, and nothing past the span, where it is hidden.
pub fn fade(bearing: f32) -> Option<f32> {
    let reach = bearing.abs() / SPAN;
    (reach <= 1.0).then(|| ((1.0 - reach) / FADE).min(1.0))
}

/// Returns the row of a mark at `at` pixels along the bar, for marks taken from left to right:
/// the first row whose last mark ends `APART` before it, else the last row. `ends` holds the
/// place of the last mark of each row and starts at minus infinity.
pub fn row(at: f32, ends: &mut [f32; ROWS]) -> usize {
    let free = ends.iter().position(|end| at - end >= APART);
    let chosen = free.unwrap_or(ROWS - 1);
    ends[chosen] = at;
    chosen
}
