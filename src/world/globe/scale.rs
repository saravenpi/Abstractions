//! The local ground from physics: how the pull and the size of a planet change its hills and its islands.

use super::elevation::tune::TUNED_RADIUS;
use super::relief::weight;
use crate::universe::HOME_GRAVITY;

/// Smallest and largest factor on the height of the mountains.
const GAIN: (f32, f32) = (0.6, 1.6);

/// Returns the factor on the height of the mountains of the local ground and the factor on the frequency
/// of its continents, for a planet of `radius` metres with a surface gravity of `gravity`.
///
/// Weaker pull lets mountains stand higher, as the square root of the home gravity over the gravity,
/// within 0.6 to 1.6. On a world the local ground shapes alone, its continents are read at the tuned
/// radius over the radius times their frequency, so they keep the same angular size on every such planet.
/// As the plates take over the continents, by [`weight`], the factor goes to 1: under the plates the local
/// ground keeps its size in metres on every planet.
pub fn scale(radius: f64, gravity: f64) -> (f32, f64) {
    let gain = ((HOME_GRAVITY / gravity).sqrt() as f32).clamp(GAIN.0, GAIN.1);
    let angular = TUNED_RADIUS / radius;
    (gain, angular + (1.0 - angular) * f64::from(weight(radius)))
}
