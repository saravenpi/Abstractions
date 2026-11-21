//! Moons in the ship's view of a system: bodies like the planets, listed after them, and the
//! point beside a moon where a route to it ends.

use bevy::math::DVec3;

use super::pilot::Goal;
use super::route::{ARRIVAL, shell};
use super::wells::PLANET_LOCK;
use crate::computer::Target;
use crate::universe::{PlanetSpec, SystemSpec, Universe};

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

/// Where a route to a moon ends: on the line from its planet through the moon, beyond the moon,
/// or anywhere near enough to the moon and clear of its planet.
#[derive(Debug, Clone, PartialEq)]
pub struct Standoff {
    /// The moon, kept from when the route was resolved so its place is not looked up again.
    pub moon: PlanetSpec,
    /// Metres from the centre of the moon to the point, away from the planet.
    pub gap: f64,
    /// Metres from the centre of the planet under which the ship is in its lock or its air.
    pub shelter: f64,
}

impl Standoff {
    /// Returns the standoff of `moon`, whose planet is `host`. The point is outside the lock
    /// sphere of the moon by the arrival share of planets, and as far from the planet as the
    /// lock sphere or the top of the air of the planet, whichever is farther, by the same share.
    pub fn of(moon: &PlanetSpec, host: &PlanetSpec) -> Self {
        let own = shell::radius(moon);
        let shelter = (host.radius * PLANET_LOCK).max(host.radius + host.atmosphere_height);
        Self {
            moon: moon.clone(),
            gap: own.max(shelter * ARRIVAL - moon.orbit_radius),
            shelter,
        }
    }

    /// Returns the offset at `time` seconds from the centre of the moon to the point.
    pub fn offset(&self, time: f64) -> DVec3 {
        self.moon.orbit().at(time).normalize_or_zero() * self.gap
    }

    /// Returns the point at `time` seconds, in metres from the star.
    pub fn point(&self, time: f64) -> DVec3 {
        self.moon.position(time) + self.offset(time)
    }

    /// Returns true when a ship at `pos` metres from the star has arrived at `time` seconds:
    /// within a small share of the lock radius of the moon from the point, or anywhere within
    /// the arrival distance of the moon that is clear of the lock and the air of its planet.
    /// The second keeps a ship that the warp drops out on the near side of the moon there,
    /// instead of spooling again for the far side.
    pub fn reached(&self, pos: DVec3, time: f64) -> bool {
        let lock = shell::lock(&self.moon);
        let host = self.moon.host.map_or(DVec3::ZERO, |host| host.at(time));
        let near = pos.distance(self.moon.position(time)) <= shell::radius(&self.moon);
        pos.distance(self.point(time)) <= lock * NEAR || (near && pos.distance(host) > self.shelter)
    }
}

/// Returns the goal of a route to `moon`, whose planet is `host`: the point beside it, or the
/// space around the moon clear of its planet, see [`Standoff::reached`].
pub fn goal(moon: &PlanetSpec, host: &PlanetSpec) -> Goal {
    let lock = shell::lock(moon);
    Goal {
        target: Target::Body(moon.id),
        lock,
        arrival: lock * NEAR,
        beyond: Some(Standoff::of(moon, host)),
    }
}
