//! Gravity wells: the bodies that pull on the ship and lock its warp drive.

use bevy::math::DVec3;

use super::frames::{hill_radius, planet_velocity};
use crate::universe::{PlanetSpec, StarSpec, SystemSpec};

/// Radii from a planet centre inside which the warp drive is locked: just outside its surface,
/// so that the lock does not grow into millions of kilometres with the size of the planet.
pub const PLANET_LOCK: f64 = 1.2;

/// Radii from a star centre inside which the warp drive is locked.
pub const STAR_LOCK: f64 = 5.0;

/// One body, in the frame of the ship.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Well {
    /// Centre in metres.
    pub pos: DVec3,
    /// Velocity in metres per second.
    pub vel: DVec3,
    /// Gravitational parameter in cubic metres per second squared.
    pub mu: f64,
    /// Radius of the body in metres.
    pub radius: f64,
    /// Distance from the centre in metres inside which the warp drive is locked.
    pub lock: f64,
    /// Radius in metres of the Hill sphere, inside which the body rules the space around it
    /// against what it goes round. Infinite for a star.
    pub hill: f64,
}

impl Well {
    /// Returns the well of a star placed at `pos`.
    pub fn star(star: &StarSpec, pos: DVec3) -> Self {
        Self {
            pos,
            vel: DVec3::ZERO,
            mu: star.mu(),
            radius: star.radius,
            lock: star.radius * STAR_LOCK,
            hill: f64::INFINITY,
        }
    }

    /// Returns the well of a planet at `time` seconds, in the frame of its star.
    pub fn planet(planet: &PlanetSpec, time: f64) -> Self {
        Self {
            pos: planet.position(time),
            vel: planet_velocity(planet, time),
            mu: planet.mu(),
            radius: planet.radius,
            lock: planet.radius * PLANET_LOCK,
            hill: hill_radius(planet),
        }
    }

    /// Returns the distance from the centre to `pos` as a share of the Hill radius: 0 at the
    /// centre, 1 at the edge of the Hill sphere.
    pub fn reach(&self, pos: DVec3) -> f64 {
        self.pos.distance(pos) / self.hill
    }

    /// Returns the metres from `pos` to the edge of the lock sphere, negative inside it.
    pub fn clearance(&self, pos: DVec3) -> f64 {
        self.pos.distance(pos) - self.lock
    }
}

/// Returns the wells of a system at `time` seconds in the frame of its star: the star, then the planets.
pub fn system_wells(system: &SystemSpec, time: f64) -> Vec<Well> {
    let star = Well::star(&system.star, DVec3::ZERO);
    let planets = system.planets.iter().map(|p| Well::planet(p, time));
    std::iter::once(star).chain(planets).collect()
}

/// Returns the pull of every well on a body at `pos`, in metres per second squared.
///
/// Each well is taken `elapsed` seconds after the moment its position was sampled.
pub fn gravity(wells: &[Well], pos: DVec3, elapsed: f64) -> DVec3 {
    let pull = |well: &Well| {
        let to = well.pos + well.vel * elapsed - pos;
        let d2 = to.length_squared().max(well.radius * well.radius * 0.25);
        to * (well.mu / (d2 * d2.sqrt()))
    };
    wells.iter().map(pull).sum()
}

/// Returns the well whose surface is closest to `pos`: the body the ship is beside. Unlike
/// [`nearest`], a planet orbiting inside the lock sphere of a large star wins when the ship is
/// near it.
pub fn beside(wells: &[Well], pos: DVec3) -> Option<&Well> {
    let gap = |well: &Well| well.pos.distance(pos) - well.radius;
    wells.iter().min_by(|a, b| gap(a).total_cmp(&gap(b)))
}

/// Returns the well whose lock sphere is closest to `pos`, if there is any well.
pub fn nearest(wells: &[Well], pos: DVec3) -> Option<&Well> {
    wells
        .iter()
        .min_by(|a, b| a.clearance(pos).total_cmp(&b.clearance(pos)))
}
