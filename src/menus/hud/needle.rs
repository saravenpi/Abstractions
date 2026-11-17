//! Where a mark of the compass sits: the angle to its target and its place along the bar.

use bevy::prelude::*;

/// Width of the bar in pixels.
pub const WIDTH: f32 = 360.0;

/// Angle in radians from the middle of the bar to either end.
pub const SPAN: f32 = 1.2;

/// Least gap in pixels between the two marks, so their distances stay readable.
const APART: f32 = 56.0;

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

/// Returns the place in pixels along the bar of the ship mark at `slot`, moved aside when the
/// home mark at `home` would sit under it, as when both wait at the same end.
pub fn beside(slot: f32, home: Option<f32>) -> f32 {
    let at = slot * WIDTH;
    match home.map(|home| home * WIDTH) {
        Some(taken) if (at - taken).abs() < APART => taken + APART * (0.5 - taken / WIDTH).signum(),
        _ => at,
    }
}
