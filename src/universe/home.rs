//! The home system: where every new game starts, generated like any other and then pinned.

use std::f64::consts::TAU;

use bevy::math::{DVec3, IVec3, Vec3};

use super::hash::{self, Rng};
use super::ids::{BodyId, GalaxyId, StarId};
use super::record::PlanetRecord;
use super::spec::{GRAVITATION, PlanetSpec, StarClass, StarSpec};
use super::system::ORBIT;
use super::zone::FLUX_ORBIT;
use super::{SECTOR_GLY, galaxy, names, planet};

/// Sea level radius of the home planet in metres.
pub const HOME_RADIUS: f64 = 6000.0;

/// Surface gravity of the home planet in metres per second squared, the gravity the player falls with.
pub const HOME_GRAVITY: f64 = 22.0;

/// Seconds in one day of the home planet, the length of the day clock cycle.
pub const HOME_DAY_SECONDS: f64 = 1200.0;

/// Rank of the home planet around the home star.
pub(super) const HOME_INDEX: u8 = 2;

/// Fewest planets around the home star: two inside the home planet and two or more beyond it,
/// so the first voyage always has a neighbour further out to go to.
pub(super) const HOME_PLANETS: u8 = 5;

/// Grid cell of the home galaxy, the one cell that always holds a galaxy.
pub(super) const HOME_GALAXY: GalaxyId = GalaxyId(IVec3::ZERO);

/// Gravitational parameter of the home star: a planet 4e7 m away takes 24 home days to go round.
const HOME_MU: f64 = 3.046e15;

/// Mass of the home star in kilograms, the unit of the star class table.
pub(super) const HOME_STAR_MASS: f64 = HOME_MU / GRAVITATION;

/// Radius of the home star in metres, the unit of the star class table.
pub(super) const HOME_STAR_RADIUS: f64 = 1.85e5;

/// Surface temperature of the home star in kelvin, the unit star temperatures are scaled from.
pub(super) const HOME_STAR_TEMPERATURE: f64 = 5780.0;

/// Returns the sector of the home star in the home galaxy of radius `radius` game light-years.
///
/// The sector lies on the middle plane of the disc, between 40 and 60 percent of the radius.
pub(super) fn sector(galaxy_seed: u64, radius: f64) -> IVec3 {
    let mut rng = Rng::new(hash::child(galaxy_seed, 4));
    let (reach, angle) = (rng.range(0.4, 0.6) * radius, rng.range(0.0, TAU));
    let at = DVec3::new(angle.cos(), 0.0, angle.sin()) * reach / SECTOR_GLY;
    at.floor().as_ivec3()
}

/// Returns the identifier of the home planet of a universe.
pub(super) fn body(universe: u64) -> BodyId {
    let shape = galaxy::shape(universe, HOME_GALAXY);
    let star = StarId {
        galaxy: HOME_GALAXY,
        sector: shape.and_then(|s| s.home).unwrap_or_default(),
        index: 0,
    };
    BodyId {
        star,
        index: HOME_INDEX,
    }
}

/// Turns the first star of the home sector into the home star, Vereya: a yellow star of unit mass and light at 5,780 K.
pub(super) fn star(star: StarSpec) -> StarSpec {
    StarSpec {
        name: names::HOME_STAR_NAME.to_string(),
        class: StarClass::G,
        mass: HOME_STAR_MASS,
        luminosity: 1.0,
        radius: HOME_STAR_RADIUS,
        temperature: HOME_STAR_TEMPERATURE,
        color: Vec3::new(1.0, 0.96, 0.88),
        ..star
    }
}

/// Returns the orbit radius in metres of a planet of the home system, from the place `band` its
/// draw gave it in the span of orbits, 0 to 1 on the log scale, and the place `home` the home
/// planet drew.
///
/// The home planet goes where its star, of luminosity 1, gives a flux of 1: [`FLUX_ORBIT`]. So
/// the temperate record it is pinned to is the one its orbit earns, and no planet further out
/// is ever warmer. The span of orbits inside it and the span beyond it are stretched or
/// squeezed evenly to fit, so the planets keep their order and their orbits never cross.
pub(super) fn orbit(band: f64, home: f64) -> f64 {
    if band == home {
        return FLUX_ORBIT;
    }
    let span = ORBIT.1 / ORBIT.0;
    let lit = (FLUX_ORBIT / ORBIT.0).ln() / span.ln();
    let at = match band < home {
        true => band * lit / home,
        false => lit + (band - home) * (1.0 - lit) / (1.0 - home),
    };
    ORBIT.0 * span.powf(at)
}

/// Turns a generated planet into the home planet, Kodya: fixed size, gravity, day and record, and the world seed.
///
/// Its record is [`PlanetRecord::TEMPERATE`], the same temperate world for every seed, and its
/// orbit, see [`orbit`], is the one that gives it the flux of that record.
pub(super) fn planet(planet: PlanetSpec, world: u64) -> PlanetSpec {
    let [atmosphere_height, cloud_base, cloud_top] = planet::air(HOME_RADIUS);
    PlanetSpec {
        seed: world,
        name: names::HOME_PLANET_NAME.to_string(),
        radius: HOME_RADIUS,
        gravity: HOME_GRAVITY,
        day_seconds: HOME_DAY_SECONDS,
        atmosphere_height,
        cloud_base,
        cloud_top,
        record: PlanetRecord::TEMPERATE,
        ..planet
    }
}
