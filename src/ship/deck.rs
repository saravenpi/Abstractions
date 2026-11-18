//! Gas giants: no ground under the air. The flight assist keeps the ship on a floor a little
//! above the cloud deck, and under the deck the air thickens until it floats the ship back up,
//! with or without the assist.

use bevy::math::DVec3;

use super::body::Ship;
use super::gear::stow;
use super::landing::{Contact, GEAR_DROP, contact};
use crate::universe::{PlanetClass, PlanetSpec};

/// Height in metres of the gear above the cloud deck, the sea level radius of a giant, under
/// which the flight assist does not let the ship descend.
pub const FLOOR: f64 = 100.0;

/// Scale heights under the deck past which the air stops thickening, so that its density stays
/// a finite number however deep a ship is pushed.
pub const DEEPEST: f64 = 20.0;

/// Slowest and fastest climb in metres per second the assist keeps under the floor of a giant,
/// and the seconds of that climb it keeps between the ship and the floor, so that it slows down
/// as it reaches the floor instead of shooting past it.
const CLIMB: (f64, f64, f64) = (1.0, 100.0, 4.0);

/// Seconds in which the assist gets to the climb it keeps under the floor of a giant.
const CLIMB_SETTLE: f64 = 0.5;

/// Returns true when `planet` has no ground to stand on: a gas giant.
pub fn giant(planet: &PlanetSpec) -> bool {
    planet.record.class == PlanetClass::GasGiant
}

/// Returns true when a ship over `planet`, or nowhere, may land and let the player out.
pub fn grounded(planet: Option<&PlanetSpec>) -> bool {
    !planet.is_some_and(giant)
}

/// Returns what the ground and the sea of `planet` do to the ship. Over a giant nothing
/// touches it, and the radar reads the height of the gear above the floor.
pub fn touch(ship: &Ship, planet: &PlanetSpec, ground: &dyn Fn(DVec3) -> f64) -> Contact {
    if !giant(planet) {
        return contact(ship, planet, ground);
    }
    Contact {
        radar: ship.pos.length() - planet.radius - FLOOR - GEAR_DROP,
        ..Contact::default()
    }
}

/// Returns what the ground and the sea of `planet` do to the ship over a step of `h` seconds,
/// after giving it the hits of the step: they change its velocity and its spin at once and set
/// a ship that is already in the ground back on it. The gear moves with the height it reads.
pub fn meet(
    ship: &mut Ship,
    planet: &PlanetSpec,
    ground: &dyn Fn(DVec3) -> f64,
    h: f64,
) -> Contact {
    let touch = touch(ship, planet, ground);
    (ship.vel, ship.spin) = (ship.vel + touch.kick, ship.spin + touch.twist);
    ship.pos += touch.normal * touch.sunk;
    stow(ship, touch.radar, h);
    touch
}

/// Returns `down`, the weight of the ship at `pos` over `planet`, with the lift of the air of
/// density `air`. Under the deck of a giant the air holds the ship up in proportion to how
/// much denser than at the deck it is: the ship floats where the air is twice as dense, and
/// no push of its engines takes it much deeper. Elsewhere `down` is returned as it is.
pub fn weigh(planet: &PlanetSpec, down: DVec3, pos: DVec3, air: f64) -> DVec3 {
    let deck = super::drag::density(planet, 0.0);
    if !giant(planet) || deck <= 0.0 {
        return down;
    }
    let pull = planet.mu() / pos.length_squared();
    down + pos.normalize() * (pull * (air / deck - 1.0).max(0.0))
}

/// Takes from the velocity of an assisted ship over a giant the part of its descent that would
/// carry the gear under the floor within the next `h` seconds. A ship already under it can
/// only rise; it is never moved.
pub fn floor(ship: &mut Ship, planet: &PlanetSpec, h: f64) {
    if !ship.assist || !giant(planet) {
        return;
    }
    let up = ship.pos.normalize();
    let room = (ship.pos.length() - planet.radius - FLOOR - GEAR_DROP).max(0.0);
    let sink = -ship.vel.dot(up) - room / h;
    if sink > 0.0 {
        ship.vel += up * sink;
    }
}

/// Returns the push of the engines of an assisted ship under the floor of `planet`, a giant,
/// from `push`, the push of the speed hold. `hold` is the weight, the drag and the lift of the
/// air, as accelerations; the speed hold cancels the first two.
///
/// Under the floor the assist leaves the vertical to the lift of the air: it cancels the weight,
/// and pushes up only where the air lifts the ship slower than the climb of [`CLIMB`] against
/// the drag, never down. The air then floats the ship up to the floor, the engines take over
/// near the deck where the air no longer lifts, and the climb eases onto the floor. Anywhere else
/// `push` is returned as it is.
pub fn buoyed(planet: &PlanetSpec, ship: &Ship, push: DVec3, hold: [DVec3; 3]) -> DVec3 {
    let below = planet.radius + FLOOR + GEAR_DROP - ship.pos.length();
    if !ship.assist || !giant(planet) || below <= 0.0 {
        return push;
    }
    let up = ship.pos.normalize();
    let [pull, lost, lift] = hold.map(|accel| accel.dot(up));
    let short = (below / CLIMB.2).clamp(CLIMB.0, CLIMB.1) - ship.vel.dot(up);
    let climb = short / CLIMB_SETTLE - lost - lift;
    push - up * push.dot(up) + up * (climb.max(0.0) - pull)
}
