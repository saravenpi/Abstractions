//! The home system: the fixed star and planets every new game starts in.

use bevy::math::{DVec3, IVec3, Vec3};

use super::ids::{BodyId, GalaxyId, StarId};
use super::spec::{GRAVITATION, GalaxyKind, GalaxySpec, PlanetSpec, StarClass, StarSpec};
use super::{GALAXY_CELL, GLY};

/// Sea level radius of the home planet in metres.
pub const HOME_RADIUS: f64 = 6000.0;

/// Surface gravity of the home planet in metres per second squared, the gravity the player falls with.
pub const HOME_GRAVITY: f64 = 22.0;

/// Seconds in one day of the home planet, the length of the day clock cycle.
pub const HOME_DAY_SECONDS: f64 = 1200.0;

/// Rank of the home planet around the home star.
const HOME_INDEX: u8 = 2;

/// Gravitational parameter of the home star, which gives the home planet a year of 24 days.
const HOME_MU: f64 = 3.046e15;

/// Name, radius, gravity, day length and orbit radius of each home planet, ordered outward.
const PLANETS: [(&str, f64, f64, f64, f64); 4] = [
    ("Cinder", 3000.0, 9.0, 2600.0, 1.4e7),
    ("Dune", 4500.0, 15.0, 1700.0, 2.6e7),
    ("Haven", HOME_RADIUS, HOME_GRAVITY, HOME_DAY_SECONDS, 4.0e7),
    ("Frost", 12000.0, 30.0, 900.0, 9.0e7),
];

/// Returns the identifier of the home star.
pub fn home_star() -> StarId {
    StarId {
        galaxy: GalaxyId(IVec3::ZERO),
        sector: IVec3::ZERO,
        index: 0,
    }
}

/// Returns the identifier of the home planet.
pub fn home_body() -> BodyId {
    BodyId {
        star: home_star(),
        index: HOME_INDEX,
    }
}

/// Returns the home galaxy for a universe seed.
pub fn galaxy(seed: u64) -> GalaxySpec {
    GalaxySpec {
        id: GalaxyId(IVec3::ZERO),
        seed,
        name: "Home".to_string(),
        kind: GalaxyKind::Spiral,
        arms: 2,
        radius: 2000.0 * GLY,
        center: DVec3::splat(GALAXY_CELL * 0.5),
    }
}

/// Returns the home star for a universe seed.
pub fn star(seed: u64) -> StarSpec {
    StarSpec {
        id: home_star(),
        seed,
        name: "Sol".to_string(),
        class: StarClass::G,
        mass: HOME_MU / GRAVITATION,
        luminosity: 1.0,
        radius: 1.85e5,
        color: Vec3::new(1.0, 0.96, 0.88),
        pos: DVec3::new(0.31, 0.5, 0.67) * GLY,
    }
}

/// Returns the planets of the home star for a universe seed, the home planet carrying the seed itself.
pub fn planets(seed: u64) -> Vec<PlanetSpec> {
    let planet =
        |(i, &(name, radius, gravity, day, orbit)): (usize, &(&str, f64, f64, f64, f64))| {
            let index = i as u8;
            let own = seed.wrapping_add(u64::from(index).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            PlanetSpec {
                id: BodyId {
                    star: home_star(),
                    index,
                },
                seed: if index == HOME_INDEX { seed } else { own },
                name: name.to_string(),
                radius,
                gravity,
                day_seconds: day,
                orbit_radius: orbit,
                orbit_seconds: std::f64::consts::TAU * (orbit * orbit * orbit / HOME_MU).sqrt(),
                orbit_phase: 1.7 * f64::from(index),
                atmosphere_height: 4000.0,
                cloud_base: 900.0,
                cloud_top: 1500.0,
            }
        };
    PLANETS.iter().enumerate().map(planet).collect()
}
