//! Moons: the bodies that orbit a planet, each a planet spec of its own with its host's orbit.

use std::f64::consts::TAU;

use super::giant::{GIANT_GRAVITY, GIANT_RADIUS};
use super::hash::{self, Rng};
use super::hill::{self, INNERMOST, Place};
use super::home::{HOME_GRAVITY, HOME_RADIUS};
use super::planet::{self, PLANET_RADIUS, RECORD_STREAM};
use super::record::PlanetRecord;
use super::spec::{PlanetSpec, StarSpec};
use super::{BodyId, Universe, names};

/// Inner heat, in units of the starlight the home planet gets, of a moon on the innermost
/// orbit of the most massive gas giant: enough to melt its rock.
pub(super) const TIDE: f64 = 600.0;

/// Gravitational parameter of the most massive gas giant, the one [`TIDE`] is given for.
const HEAVIEST: f64 = GIANT_GRAVITY.1 * HOME_GRAVITY * GIANT_RADIUS.1 * GIANT_RADIUS.1;

/// Returns how many moons the planet has: as many as [`hill::places`] fits around it.
pub(super) fn count(star: &StarSpec, planet: &PlanetSpec) -> u8 {
    hill::places(star, planet).1 as u8
}

/// Returns the moons of `planet`, a planet of `star`, ordered outward. Every moon has
/// `id.moon` from 1 up and `host` set to the orbit of its planet.
///
/// There are as many as `planet.moons` says, which is 0 for a moon and for the home planet.
pub(super) fn of(star: &StarSpec, planet: &PlanetSpec) -> Vec<PlanetSpec> {
    let (places, used) = hill::places(star, planet);
    let kept = used.min(usize::from(planet.moons));
    let ranks = 1..=kept as u8;
    ranks
        .zip(places)
        .map(|(rank, place)| build(star, planet, rank, place))
        .collect()
}

/// Returns moon number `rank` of `planet`, a planet of `star`, the same spec [`of`] lists at
/// `rank - 1`, or nothing when it has no such moon. Only that moon is built: the orbits are
/// placed once and the moons inside it are skipped.
pub(super) fn ranked(star: &StarSpec, planet: &PlanetSpec, rank: u8) -> Option<PlanetSpec> {
    let (places, used) = hill::places(star, planet);
    let kept = used.min(usize::from(planet.moons));
    let index = usize::from(rank)
        .checked_sub(1)
        .filter(|index| *index < kept)?;
    Some(build(star, planet, rank, places[index]))
}

/// Returns the inner heat of a moon on an orbit of `orbit` metres around `planet`, in units of
/// the starlight the home planet gets.
///
/// Tides heat a moon as the square of the mass of its planet over the sixth power of its orbit
/// radius: [`TIDE`] on the innermost orbit, 4 radii, of the most massive giant, 64 times less at
/// twice that distance, and under 1 around any solid planet.
pub(super) fn tide(planet: &PlanetSpec, orbit: f64) -> f64 {
    let mass = planet.mu() / HEAVIEST;
    TIDE * mass * mass * (INNERMOST * planet.radius / orbit).powi(6)
}

/// Builds moon number `rank` of `planet` from its place.
///
/// A moon is a small planet: its gravity follows its size as a planet's does, it turns once per
/// orbit, so it always shows the same face to its planet, and its record is lit by the star
/// at the orbit of its planet and warmed by [`tide`]. Its day is its orbit with the sign
/// flipped: an orbit turns from +X toward +Z, the opposite way to a positive spin, so a moon
/// spins the negative way to keep its near side on its planet. It runs from about -800 s close
/// to a light planet to -220,000 s far out around a heavy one.
fn build(star: &StarSpec, planet: &PlanetSpec, rank: u8, place: Place) -> PlanetSpec {
    let mut rng = Rng::new(place.seed);
    let span = (PLANET_RADIUS.1 / PLANET_RADIUS.0).ln();
    let earths = planet::earths((place.radius / PLANET_RADIUS.0).ln() / span, rng.unit());
    let seconds = planet::orbit_seconds(planet.mu(), place.orbit);
    let [atmosphere_height, cloud_base, cloud_top] = planet::air(place.radius);
    let body = [earths, place.radius / HOME_RADIUS];
    let draws = hash::child(place.seed, RECORD_STREAM);
    let heat = tide(planet, place.orbit);
    PlanetSpec {
        id: BodyId {
            moon: rank,
            ..planet.id
        },
        seed: place.seed,
        name: names::moon(&planet.name, rank),
        radius: place.radius,
        gravity: earths * HOME_GRAVITY,
        day_seconds: -seconds,
        orbit_radius: place.orbit,
        orbit_seconds: seconds,
        orbit_phase: rng.range(0.0, TAU),
        atmosphere_height,
        cloud_base,
        cloud_top,
        record: PlanetRecord::heated(star, planet.orbit_radius, body, draws, heat),
        host: Some(planet.orbit()),
        moons: 0,
        rings: None,
    }
}

impl Universe {
    /// Returns the moons of the planet that `id` is or orbits, ordered outward.
    pub fn moons(&self, id: BodyId) -> Vec<PlanetSpec> {
        let system = self.system(id.star);
        let last = system.planets.len() - 1;
        of(
            &system.star,
            &system.planets[usize::from(id.index).min(last)],
        )
    }
}
