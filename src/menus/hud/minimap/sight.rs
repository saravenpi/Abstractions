//! What the ground view shows for one pose: how the image sits, and where the home base, the
//! ship and north are on the map.
//!
//! Everything goes through the chart of one patch, so the image and the marks agree.

use bevy::math::{DVec3, Vec2};

use super::patch::Patch;
use super::pose::Pose;
use super::view::{Fit, fit, heading, pin, rim, to_view};

/// Step along a direction, as a share of the radius of the planet, that tells which way the
/// direction points on the chart.
const STEP: f64 = 1e-3;

/// The ground view for one pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sight {
    /// How the image of the ground sits in the frame.
    pub fit: Fit,
    /// Map point of the north mark, on the rim.
    pub north: Vec2,
    /// Map point of the home base, when it stands on this planet.
    pub home: Option<Vec2>,
    /// Map point of the ship, while the player is on foot on its planet.
    pub ship: Option<Vec2>,
}

/// Returns the ground view of `pose` through `patch`, for a map that shows `radius` metres to
/// each side, with the home base at `home` in the frame of the planet. The heading is up, and
/// a mark past the rim waits on it.
pub fn sight(pose: &Pose, patch: &Patch, radius: f32, home: Option<DVec3>) -> Sight {
    let dir = pose.at.normalize();
    let at = patch.locate(dir);
    let toward = |along: DVec3| patch.locate((dir + along * STEP).normalize()) - at;
    let turn = heading(toward(pose.ahead)).unwrap_or(0.0);
    let north = (DVec3::Y - dir * dir.y).try_normalize().unwrap_or(DVec3::X);
    let mark = |pos: DVec3| spot(pose, patch, radius, pos);
    Sight {
        fit: fit(patch.half(), at, turn, radius),
        north: rim(toward(north), turn),
        home: home.map(mark),
        ship: pose.ship.map(mark),
    }
}

/// Returns the map point of `pos`, in metres from the centre of the planet, seen from `pose`
/// through `patch` on a map that shows `radius` metres to each side with the heading up. A
/// point past the rim waits on it.
pub fn spot(pose: &Pose, patch: &Patch, radius: f32, pos: DVec3) -> Vec2 {
    let dir = pose.at.normalize();
    let at = patch.locate(dir);
    let ahead = patch.locate((dir + pose.ahead * STEP).normalize()) - at;
    let turn = heading(ahead).unwrap_or(0.0);
    pin(to_view(patch.locate(pos.normalize()) - at, turn, radius)).0
}
