//! What lies under the boots: the ground, or the sea on a planet that has one.

use crate::world::Globe;

/// Returns the free height under the soles in metres, and whether the sea is what lies nearest.
///
/// `ground` is the free height down to the ground and `height` the height of the soles over
/// the sea datum of `globe`. A planet without a sea has only ground: a dry basin below the
/// datum is never wet, so the boots raise dust there and no spray.
pub fn under(globe: &Globe, ground: f32, height: f32) -> (f32, bool) {
    let sea = height - globe.sea_level();
    (ground.min(sea).max(0.0), sea < ground)
}
