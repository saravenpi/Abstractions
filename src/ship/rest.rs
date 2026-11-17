//! Touchdown: damage from hard landings, and the rest state of a ship that holds on its gear.

use super::assist::NEUTRAL;
use super::body::Ship;
use super::flight::Controls;
use super::landing::{Contact, FRICTION};
use bevy::math::DVec3;

/// Speed in metres per second under which a ship on its gear comes to rest.
const REST_SPEED: f64 = 0.6;

/// Spin in radians per second under which a ship on its gear comes to rest.
const REST_SPIN: f64 = 0.2;

/// Share of the friction limit a slope may reach for the ship to rest on it.
const REST_SLOPE: f64 = 0.9;

/// Metres per second above the safe speed that take the damage from none to full.
const DAMAGE_SPAN: f64 = 30.0;

/// Seconds the gear damps weakly after a landing above the safe speed.
const BOUNCE_TIME: f64 = 1.5;

/// Returns true when the controls ask a landed ship to move again.
pub fn wakes(ship: &Ship, controls: &Controls) -> bool {
    ship.throttle.abs() >= NEUTRAL || controls.lift > 0.0
}

/// Seconds a ship must stay slow and held before it comes to rest.
const REST_TIME: f64 = 0.5;

/// Returns true when the ship is idle, slow and held by the ground or by the sea.
fn rests(ship: &Ship, contact: &Contact, controls: &Controls) -> bool {
    let slow = ship.vel.length() < REST_SPEED && ship.spin.length() < REST_SPIN;
    let held = contact.gear == 3 && contact.slope < FRICTION * REST_SLOPE;
    let afloat = contact.wet && !contact.ground;
    !wakes(ship, controls) && slow && (held || afloat)
}

/// Books a step of contact on the ship and returns the touchdown speed, zero without a new touchdown.
///
/// A touchdown above the safe speed adds damage and makes the gear bounce. A ship that stays slow and
/// held for half a second stops dead.
pub fn settle(ship: &mut Ship, contact: &Contact, controls: &Controls, h: f64) -> f64 {
    let landing = contact.ground && !ship.touching;
    ship.touching = contact.ground;
    ship.bounce = (ship.bounce - h).max(0.0);
    if landing && contact.excess > 0.0 {
        ship.damage = (ship.damage + contact.excess / DAMAGE_SPAN).min(1.0);
        ship.bounce = BOUNCE_TIME;
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
