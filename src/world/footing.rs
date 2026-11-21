//! The height a saved place may have: a body can be on the ground, in the sea or in the air
//! over a planet, and nowhere else.

use bevy::math::DVec3;

use super::Globe;
use crate::saves::Place;

/// Depth in metres under the ground down to which a saved body centre is left where it is: the
/// ground of a chunk differs a little from the ground read here, and a body sunk less than this
/// is lifted by the player's own rescue.
const SUNK: f64 = 2.5;

/// Height in metres above the ground or the sea at which a body put back stands.
const STAND: f64 = 1.5;

/// Returns `place` at a height a body can be at on `globe`: itself when its height is a number
/// between the ground and the distance where the world stops drawing the planet, the air
/// included. Else the same direction on the ground, or on the sea where the ground is under it.
/// A place that is not [`Place::pointed`] is returned as it is.
pub(super) fn grounded(place: Place, globe: &Globe) -> Place {
    let dir = DVec3::from(place.dir).try_normalize();
    let Some(dir) = dir.filter(|_| place.pointed()) else {
        return place;
    };
    let ground = f64::from(globe.height(dir));
    let top = globe.handover() - globe.radius();
    let high = place.altitude;
    if high.is_finite() && high >= ground - SUNK && high <= top {
        return place;
    }
    Place {
        altitude: ground.max(f64::from(globe.sea_level())) + STAND,
        ..place
    }
}
