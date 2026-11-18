//! Gas giants: no ground under the air. The flight assist keeps the ship on a floor a little
//! above the cloud deck, and under the deck the air thickens until it floats the ship back up.

use bevy::math::DVec3;

use super::body::Ship;
use super::landing::{Contact, GEAR_DROP, contact};
use crate::universe::{PlanetClass, PlanetSpec};

/// Height in metres of the gear above the cloud deck, the sea level radius of a giant, under
/// which the flight assist does not let the ship descend.
pub const FLOOR: f64 = 100.0;

/// Scale heights under the deck past which the air stops thickening, so that its density stays
/// a finite number however deep a ship is pushed.
pub const DEEPEST: f64 = 20.0;

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
