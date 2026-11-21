//! The height a saved place may have: a body on foot can be on the ground, in the sea or in
//! the air of a planet, and nowhere else. A body seated in the ship is where the ship is.

use bevy::math::DVec3;

use super::Globe;
use crate::saves::Place;

/// Depth in metres under the ground down to which a saved body centre is left where it is: the
/// ground of a chunk differs a little from the ground read here, and a body sunk less than this
/// is lifted by the player's own rescue.
const SUNK: f64 = 2.5;

/// Height in metres above the ground or the sea at which a body put back stands.
const STAND: f64 = 1.5;

/// Height in metres above the ground or the sea up to which a body on foot is left in the air
/// whatever the air: a hop or a short fall, which the robot takes.
const HOP: f64 = 50.0;

/// Returns the height in metres above sea level of the ground in direction `dir` of `globe`,
/// and of what a body stands or floats on there: the ground, or the sea over it.
fn floors(globe: &Globe, dir: DVec3) -> (f64, f64) {
    let ground = f64::from(globe.height(dir));
    (ground, ground.max(f64::from(globe.sea_level())))
}

/// Returns `place` at a height a body on foot can be at on `globe`, whose air ends `air` metres
/// above sea level: itself when its height is a number, not under the ground, and either
/// within the air, where the wingsuit flies, or within [`HOP`] of the ground. Else the same
/// direction on the ground, or on the sea where the ground is under it: a body on foot is
/// never left in space or falling through no air. A place that is not [`Place::pointed`] is
/// returned as it is.
pub(super) fn grounded(place: Place, globe: &Globe, air: f64) -> Place {
    let dir = DVec3::from(place.dir).try_normalize();
    let Some(dir) = dir.filter(|_| place.pointed()) else {
        return place;
    };
    let (ground, floor) = floors(globe, dir);
    let high = place.altitude;
    let flown = high <= air || high <= floor + HOP;
    if high.is_finite() && high >= ground - SUNK && flown {
        return place;
    }
    Place {
        altitude: floor + STAND,
        ..place
    }
}

/// Returns `place` as the spot the world loads around for a player seated in the ship, on
/// `globe`: the same direction, no lower than the ground and no higher than just past where the
/// world stops drawing the planet. The ship says where the player really is; this only keeps
/// the loading screen over real ground or clear of the planet. A place that is not
/// [`Place::pointed`] is returned as it is.
pub(super) fn aboard(place: Place, globe: &Globe) -> Place {
    let dir = DVec3::from(place.dir).try_normalize();
    let Some(dir) = dir.filter(|_| place.pointed()) else {
        return place;
    };
    let top = (globe.handover() - globe.radius()) * 1.01;
    let low = floors(globe, dir).1 + STAND;
    let high = match place.altitude.is_nan() {
        true => top,
        false => place.altitude.clamp(low.min(top), top),
    };
    Place {
        altitude: high,
        ..place
    }
}
