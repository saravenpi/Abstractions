//! Relief from physics: how the pull and the size of a planet change its mountains and its continents.

use crate::universe::{EARTH_RADIUS, STANDARD_GRAVITY};

/// Smallest and largest factor on the height of the mountains.
const GAIN: (f32, f32) = (0.6, 1.6);

/// Returns the factor on the height of the mountains and the factor on the frequency of the continents of
/// a planet of `radius` metres with a surface gravity of `gravity`.
///
/// Weaker pull lets mountains stand higher, as the square root of standard gravity over the gravity,
/// within 0.6 to 1.6. The continents are read at the radius of the Earth over the radius times their
/// frequency, so they keep the same angular size on every planet. A planet with the size and the pull of
/// the Earth gets exactly 1 and 1.
pub fn scale(radius: f64, gravity: f64) -> (f32, f64) {
    let gain = ((STANDARD_GRAVITY / gravity).sqrt() as f32).clamp(GAIN.0, GAIN.1);
    (gain, EARTH_RADIUS / radius)
}
