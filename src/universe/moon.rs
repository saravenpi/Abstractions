//! Moons: the bodies that orbit a planet, each a planet spec of its own with its host's orbit.

use std::f64::consts::TAU;

use super::hash::{self, Rng};
use super::hill::{self, Place};
use super::planet::{self, RECORD_STREAM};
use super::record::{Body, PlanetRecord};
use super::spec::{PlanetSpec, StarSpec};
use super::units::STARLIGHT;
use super::{BodyId, Universe, air, names};

/// Heat the tides of Jupiter draw out of Io, in units of flux: about 2.5 watts per square metre,
/// the measure every other tide is scaled from.
pub(super) const IO_HEAT: f64 = 2.5 / STARLIGHT;

/// Gravitational parameter of Jupiter, the radius of Io and the radius of its orbit, in SI units.
const IO: [f64; 3] = [1.267e17, 1.8216e6, 4.217e8];

/// Returns how many moons the planet has: as many as [`hill::places`] fits around it.
pub(super) fn count(star: &StarSpec, planet: &PlanetSpec) -> u8 {
    hill::places(star, planet).1 as u8
}

/// Returns the moons of `planet`, a planet of `star`, ordered outward. Every moon has
/// `id.moon` from 1 up and `host` set to the orbit of its planet.
///
/// There are as many as `planet.moons` says, which is 0 for a moon.
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

/// Returns the heat the tides of `planet` store in the moon at `place`, in units of flux.
///
/// A moon is heated as its planet kneads it on an orbit its neighbours keep out of round. The
/// heat flow through its ground goes as the mass of the planet to the power 2.5, the cube of
/// the radius of the moon, the inverse of its orbit radius to the power 7.5 and the square of
/// its eccentricity, here [`Place::forcing`]. It is scaled from Io, see [`IO_HEAT`]: a regular
/// moon close to a heavy giant gets hundreds of times that, and a lone or captured moon none.
pub(super) fn tide(planet: &PlanetSpec, place: &Place) -> f64 {
    let mass = (planet.mu() / IO[0]).powf(2.5);
    let size = (place.radius / IO[1]).powi(3);
    IO_HEAT * place.forcing * mass * size * (IO[2] / place.orbit).powf(7.5)
}

/// Builds moon number `rank` of `planet` from its place.
///
/// A moon is a small planet. Its gravity follows its size and its matter as a planet's does.
/// Tides locked it long ago: it turns once per orbit, the same way round, so it always shows
/// the same face to its planet. Its record is lit by the star at the orbit of its planet and
/// warmed by [`tide`].
fn build(star: &StarSpec, planet: &PlanetSpec, rank: u8, place: Place) -> PlanetSpec {
    let mut rng = Rng::new(place.seed);
    let gravity = planet::gravity(place.radius, place.matter);
    let seconds = planet::orbit_seconds(planet.mu(), place.orbit);
    let body = Body {
        tide: tide(planet, &place),
        ..Body::new(gravity, place.radius, planet.orbit_radius)
    };
    let record = PlanetRecord::derive(star, &body, hash::child(place.seed, RECORD_STREAM));
    let [atmosphere_height, cloud_base, cloud_top] = air::heights(&record, gravity, place.radius);
    PlanetSpec {
        id: BodyId {
            moon: rank,
            ..planet.id
        },
        seed: place.seed,
        name: names::moon(&planet.name, rank),
        radius: place.radius,
        gravity,
        day_seconds: seconds,
        orbit_radius: place.orbit,
        orbit_seconds: seconds,
        orbit_phase: rng.range(0.0, TAU),
        atmosphere_height,
        cloud_base,
        cloud_top,
        record,
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
