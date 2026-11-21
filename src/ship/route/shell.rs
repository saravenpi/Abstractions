//! Where a route to a planet or a moon ends: its arrival shell, a sphere just outside its air that the
//! warp drive may reach, so that the flight in takes a minute or two and not a quarter of an hour.
//!
//! The lock sphere of a body, [`crate::ship::wells::PLANET_LOCK`] radii, guards the drive of a ship that
//! flies past it. A course aimed at the body itself is the one flight that must end near it, so for that
//! course the lock shrinks to [`lock`], the air with a margin or a few hundredths of the radius on an airless
//! body, and the drive drops out at [`radius`], the drop-out margin outside it. Every other body keeps its
//! lock sphere, and the shell is never inside a body or its air.

use crate::ship::warp::DROP_MARGIN;
use crate::universe::PlanetSpec;

/// Share of the radius of an airless body that the lock of its shell stands off the ground.
const CLEAR: f64 = 0.02;

/// Share of the height of the air that the lock of the shell stands off the top of it.
const AIR: f64 = 1.1;

/// Returns the radius in metres from the centre of `planet` of the lock a course aimed at it flies
/// under: outside the top of its air, and off the ground by at least [`CLEAR`] of its radius.
pub(in crate::ship) fn lock(planet: &PlanetSpec) -> f64 {
    let aired = planet.radius + planet.atmosphere_height * AIR;
    aired.max(planet.radius * (1.0 + CLEAR))
}

/// Returns the radius in metres from the centre of `planet` of its arrival shell, where the drive of a
/// course aimed at it drops out: the drop-out margin outside [`lock`].
pub(in crate::ship) fn radius(planet: &PlanetSpec) -> f64 {
    lock(planet) * (1.0 + DROP_MARGIN)
}
