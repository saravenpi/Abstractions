//! Touchdown: damage from hard landings, and the rest state of a ship held by the ground.

use super::assist::NEUTRAL;
use super::body::Ship;
use super::flight::Controls;
use super::landing::{Contact, FRICTION};
use bevy::math::DVec3;

/// Speed in metres per second under which a ship on its gear comes to rest.
pub(super) const REST_SPEED: f64 = 0.6;

/// Spin in radians per second under which a ship on its gear comes to rest.
pub(super) const REST_SPIN: f64 = 0.2;

/// Share of the friction limit a slope may reach for the ship to rest on it.
const REST_SLOPE: f64 = 0.9;

/// Metres per second above the safe speed that take the damage from none to full.
const DAMAGE_SPAN: f64 = 30.0;

/// Returns true when the controls ask a landed ship to move again: without assist the lever
/// or the rise, with it only the rise, so that a take-off is always meant.
pub fn wakes(ship: &Ship, controls: &Controls) -> bool {
    (!ship.assist && ship.throttle.abs() >= NEUTRAL) || controls.lift > 0.0
}

/// Adds to the damage of the ship what a hit `excess` metres per second above the safe speed
/// of the point that took it does. A hit at or under the safe speed does none.
pub fn harm(ship: &mut Ship, excess: f64) {
    if excess > 0.0 {
        ship.damage = (ship.damage + excess / DAMAGE_SPAN).min(1.0);
    }
}

/// Seconds a ship must stay slow and held before it comes to rest.
const REST_TIME: f64 = 0.5;

/// Returns true when the ship is idle, slow and held by the ground on three points or more,
/// gear or hull, or by the sea.
fn rests(ship: &Ship, contact: &Contact, controls: &Controls) -> bool {
    let slow = ship.vel.length() < REST_SPEED && ship.spin.length() < REST_SPIN;
    let held = contact.points >= 3 && contact.slope < FRICTION * REST_SLOPE;
    let afloat = contact.wet && !contact.ground;
    !wakes(ship, controls) && slow && (held || afloat)
}

/// Books a step of contact on the ship and returns the touchdown speed, zero without a new touchdown.
///
/// A touchdown above the safe speed adds damage. A ship that stays slow and held for half a
/// second stops dead.
pub fn settle(ship: &mut Ship, contact: &Contact, controls: &Controls, h: f64) -> f64 {
    let landing = contact.ground && !ship.touching;
    ship.touching = contact.ground;
    ship.since = [ship.since + h, 0.0][usize::from(contact.ground)];
    if landing {
        harm(ship, contact.excess);
    }
    ship.calm = if rests(ship, contact, controls) {
        ship.calm + h
    } else {
        0.0
    };
    if ship.calm >= REST_TIME {
        ship.landed = true;
        ship.vel = DVec3::ZERO;
        ship.spin = DVec3::ZERO;
    }
    if landing { contact.impact } else { 0.0 }
}

/// Metres per second a ship touches down at and rebounds at, at most, that the time it takes to come back
/// down is worked out from.
const REBOUND: f64 = 2.0;

/// Metres above the ground within which a ship that touched it is taken to be rebounding.
pub(super) const REBOUND_HEIGHT: f64 = 4.0;

/// Returns the seconds after a touch of the gear in which a ship with the stick released is still being
/// set down, under the pull of `gravity` metres per second squared: the time a rebound at [`REBOUND`] takes
/// to rise and fall, between half a second and six. In the weak gravity of a moon a bounce is long, and an
/// assist that took hold of the ship in the air again would hop it up each time it touched.
pub(super) fn bounce_time(gravity: f64) -> f64 {
    (2.0 * REBOUND / gravity.max(0.1)).clamp(0.5, 6.0)
}
