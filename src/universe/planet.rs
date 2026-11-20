//! Planets: the physical values of one planet, each inside a documented range.

use std::f64::consts::TAU;

use super::hash::{self, Rng};
use super::home::{HOME_GRAVITY, HOME_RADIUS};
use super::ids::BodyId;
use super::record::PlanetRecord;
use super::spec::{PlanetSpec, StarSpec};
use super::{air, giant, moon, names, rings};

/// Smallest and largest sea level radius of a planet in metres.
pub const PLANET_RADIUS: (f64, f64) = (3000.0, 40_000.0);

/// Weakest and strongest surface gravity of a planet in Earth gravities.
///
/// The home planet stands for 1 Earth gravity, so a planet of `g` Earth gravities pulls with
/// `g * HOME_GRAVITY` metres per second squared: [`HOME_GRAVITY`] plays the part of 9.81.
pub const PLANET_GRAVITY: (f64, f64) = (0.3, 1.8);

/// Shortest and longest day of a planet in seconds.
pub const PLANET_DAY: (f64, f64) = (600.0, 3600.0);

/// Child of the planet seed the record draws from, so the draws of the body itself never shift.
pub(super) const RECORD_STREAM: u64 = 1;

/// Returns the seconds a circular orbit of `radius` metres takes around a body of parameter `mu`.
pub fn orbit_seconds(mu: f64, radius: f64) -> f64 {
    TAU * (radius * radius * radius / mu).sqrt()
}

/// Returns the surface gravity in Earth gravities of a body at `size`, its place from 0 to 1 on
/// the log scale of [`PLANET_RADIUS`], for a draw `roll` from 0 to 1: half of it follows the size.
pub(super) fn earths(size: f64, roll: f64) -> f64 {
    let pull = 0.5 * (size + roll);
    PLANET_GRAVITY.0 + (PLANET_GRAVITY.1 - PLANET_GRAVITY.0) * pull
}

/// Builds planet number `index` of a star on an orbit of `orbit` metres.
///
/// The planet is first built solid, then turns into a gas giant when [`giant::forms`] says so
/// or when `giant` asks for one, and last gets its rings and the count of its moons. Each step
/// draws from its own child of the seed, so a planet that stays solid keeps every value it had
/// before giants, rings and moons existed.
pub(super) fn build(star: &StarSpec, index: u8, orbit: f64, seed: u64, giant: bool) -> PlanetSpec {
    let body = solid(star, index, orbit, seed);
    let body = match giant || giant::forms(star, body.record.flux, seed) {
        true => giant::grow(star, body, hash::child(seed, RECORD_STREAM)),
        false => body,
    };
    PlanetSpec {
        rings: rings::of(&body),
        moons: moon::count(star, &body),
        ..body
    }
}

/// Builds the solid planet number `index` of a star on an orbit of `orbit` metres.
///
/// Radii are spread evenly on a log scale, so small planets are the common ones, and half of
/// the gravity follows the size. The record follows from the star, the orbit, the gravity and
/// the size, with its own draws from a child of the seed, and the air heights from the record,
/// see [`air::heights`].
fn solid(star: &StarSpec, index: u8, orbit: f64, seed: u64) -> PlanetSpec {
    let mut rng = Rng::new(seed);
    let size = rng.unit();
    let radius = PLANET_RADIUS.0 * (PLANET_RADIUS.1 / PLANET_RADIUS.0).powf(size);
    let earths = earths(size, rng.unit());
    let draws = hash::child(seed, RECORD_STREAM);
    let record = PlanetRecord::derive(star, orbit, earths, radius / HOME_RADIUS, draws);
    let [atmosphere_height, cloud_base, cloud_top] =
        air::heights(&record, earths * HOME_GRAVITY, radius);
    PlanetSpec {
        id: BodyId {
            star: star.id,
            index,
            moon: 0,
        },
        seed,
        name: names::planet(&star.name, index),
        radius,
        gravity: earths * HOME_GRAVITY,
        day_seconds: rng.range(PLANET_DAY.0, PLANET_DAY.1),
        orbit_radius: orbit,
        orbit_seconds: orbit_seconds(star.mu(), orbit),
        orbit_phase: rng.range(0.0, TAU),
        atmosphere_height,
        cloud_base,
        cloud_top,
        record,
        host: None,
        moons: 0,
        rings: None,
    }
}
