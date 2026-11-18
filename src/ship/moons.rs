//! Moons in the ship's view of a system: bodies like the planets, listed after them, and the
//! point beside a moon where a route to it ends.

use bevy::math::DVec3;

use super::pilot::Goal;
use super::route::ARRIVAL;
use super::wells::PLANET_LOCK;
use crate::computer::Target;
use crate::universe::{Orbit, PlanetSpec, SystemSpec, Universe};

/// Share of the lock radius of a moon within which the ship has reached the point beside it.
/// It keeps the ship outside the lock sphere of the moon, past the margin at which a warp
/// drops out by itself.
const NEAR: f64 = 0.15;

/// Returns `system` with the moons of its planets listed after the planets, so that every body
/// the ship can feel, enter or stop beside is in `planets`. Moons are found once per system,
/// only for the planets that have some.
pub fn with_moons(universe: &Universe, mut system: SystemSpec) -> SystemSpec {
    let moons: Vec<PlanetSpec> = system
        .planets
        .iter()
        .filter(|planet| planet.moons > 0)
        .flat_map(|planet| universe.moons(planet.id))
        .collect();
    system.planets.extend(moons);
    system
}

/// Where a route to a moon ends: on the line from its planet through the moon, beyond the moon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Standoff {
    /// The orbit of the moon around its planet, which turns the line.
    pub orbit: Orbit,
    /// Metres from the centre of the moon to the point, away from the planet.
    pub gap: f64,
}

impl Standoff {
    /// Returns the standoff of `moon`, whose planet is `host`. The point is outside the lock
    /// sphere of the moon by the arrival share of planets, and as far from the planet as the
    /// lock sphere or the top of the air of the planet, whichever is farther, by the same share.
    pub fn of(moon: &PlanetSpec, host: &PlanetSpec) -> Self {
        let own = moon.radius * PLANET_LOCK * ARRIVAL;
        let shelter = (host.radius * PLANET_LOCK).max(host.radius + host.atmosphere_height);
        Self {
            orbit: moon.orbit(),
            gap: own.max(shelter * ARRIVAL - moon.orbit_radius),
        }
    }

    /// Returns the offset at `time` seconds from the centre of the moon to the point.
    pub fn offset(&self, time: f64) -> DVec3 {
        self.orbit.at(time).normalize_or_zero() * self.gap
    }
}

/// Returns the goal of a route to `moon`, whose planet is `host`: the point beside it, reached
/// within a small share of its lock radius.
pub fn goal(moon: &PlanetSpec, host: &PlanetSpec) -> Goal {
    let lock = moon.radius * PLANET_LOCK;
    Goal {
        target: Target::Body(moon.id),
        lock,
        arrival: lock * NEAR,
        beyond: Some(Standoff::of(moon, host)),
    }
}
