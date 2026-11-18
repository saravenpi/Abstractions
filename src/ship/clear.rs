//! Ground clearance of a turn: near the ground the flight assist drops any part of the stick
//! that would swing a contact point of the ship into the ground, so that no input ever turns
//! the hull into it.

use bevy::math::{DQuat, DVec3};

use super::assist::TURN_RATE;
use super::body::Ship;
use super::gear::feet;
use super::landing::{Contact, GEAR_DROP, HULL};

/// Metres every contact point keeps above the ground plane while the stick turns the ship.
pub const MARGIN: f64 = 0.5;

/// Seconds of a turn at the rate asked for that are looked ahead.
const LOOK: f64 = 0.3;

/// Returns the height in metres above the ground plane of the lowest contact point of `ship`
/// turned to `rot`, its centre held where it is.
fn lowest(ship: &Ship, touch: &Contact, rot: DQuat) -> f64 {
    let centre = (touch.radar + GEAR_DROP) * touch.normal.dot(ship.pos.normalize());
    let low = feet(ship)
        .into_iter()
        .chain(HULL)
        .map(|body| touch.normal.dot(rot * body))
        .fold(f64::INFINITY, f64::min);
    centre + low
}

/// Returns `stick` without each of its pitch, yaw and roll parts that, held for [`LOOK`]
/// seconds, would bring the lowest contact point lower and under [`MARGIN`] above the ground
/// plane. Out of reach of the ground the stick is returned as it is.
pub fn guard(ship: &Ship, touch: &Contact, stick: DVec3) -> DVec3 {
    if touch.normal == DVec3::ZERO || stick == DVec3::ZERO {
        return stick;
    }
    let now = lowest(ship, touch, ship.rot);
    let keeps = |axis: DVec3| {
        let turn = DQuat::from_scaled_axis(axis * stick * TURN_RATE * LOOK);
        let then = lowest(ship, touch, ship.rot * turn);
        then >= MARGIN || then >= now - 1e-9
    };
    let kept = [DVec3::X, DVec3::Y, DVec3::Z].map(|axis| f64::from(u8::from(keeps(axis))));
    stick * DVec3::from_array(kept)
}
