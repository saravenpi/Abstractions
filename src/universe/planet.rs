//! Planets: the physical values of one planet, each inside a documented range.

use std::f64::consts::{PI, TAU};

use super::hash::{self, Rng};
use super::ids::BodyId;
use super::record::{Body, PlanetRecord};
use super::spec::{GRAVITATION, PlanetSpec, StarSpec};
use super::spin::{self, QUALITY, SLOWEST, Turning};
use super::units::EARTH_RADIUS;
use super::{giant, moon, names, rings};

/// Smallest and largest sea level radius of a solid planet in metres: from a little under the
/// Moon to nearly twice the Earth.
pub const PLANET_RADIUS: (f64, f64) = (1.5e6, 1.2e7);

/// Lightest and densest rock a solid planet is made of, in kilograms per cubic metre before its
/// own weight squeezes it: from ice-rich rock to iron-rich rock. The Earth is at 4,050.
pub const ROCK: (f64, f64) = (3000.0, 5400.0);

/// How much denser its own weight makes a planet the size of the Earth: 36 percent. The
/// squeeze grows as the square of the radius.
const SQUEEZE: f64 = 0.36;

/// Child of the planet seed the record draws from, so the draws of the body itself never shift.
pub(super) const RECORD_STREAM: u64 = 1;

/// Atmosphere top, cloud base and cloud top of a planet the size of the Earth, in metres above
/// sea level.
const EARTH_AIR: [f64; 3] = [4000.0, 900.0, 1500.0];

/// Returns the atmosphere top, cloud base and cloud top of a planet in metres above sea level.
///
/// The three heights are 4,000, 900 and 1,500 m times the square root of the radius over the
/// radius of the Earth.
pub fn air(radius: f64) -> [f64; 3] {
    let scale = (radius / EARTH_RADIUS).sqrt();
    EARTH_AIR.map(|height| height * scale)
}

/// Returns the seconds a circular orbit of `radius` metres takes around a body of parameter `mu`.
pub fn orbit_seconds(mu: f64, radius: f64) -> f64 {
    TAU * (radius * radius * radius / mu).sqrt()
}

/// Returns the surface gravity in metres per second squared of a solid body of `radius` metres
/// made of rock of `rock` kilograms per cubic metre before it is squeezed.
///
/// The mean density is `rock * (1 + 0.36 (R / R_earth)^2)`, which matches Mercury, Mars, Venus
/// and the Earth within 4 percent, and the gravity is `4/3 pi G density R`: 9.81 for the Earth,
/// 1.3 for the smallest and lightest planet, 41 for the largest and densest.
pub fn gravity(radius: f64, rock: f64) -> f64 {
    let size = radius / EARTH_RADIUS;
    let density = rock * (1.0 + SQUEEZE * size * size);
    4.0 / 3.0 * PI * GRAVITATION * density * radius
}

/// Builds planet number `index` of a star on an orbit of `orbit` metres.
///
/// The planet is first built solid, then turns into a gas giant when [`giant::forms`] says so,
/// and last gets its rings and the count of its moons. Each step draws from its own child of
/// the seed, so a planet that stays solid keeps every value whatever giants, rings and moons do.
pub(super) fn build(star: &StarSpec, index: u8, orbit: f64, seed: u64) -> PlanetSpec {
    let body = solid(star, index, orbit, seed);
    let body = match giant::forms(star, body.record.flux, seed) {
        true => giant::grow(star, body, hash::child(seed, RECORD_STREAM)),
        false => body,
    };
    PlanetSpec {
        rings: rings::of(&body),
        moons: moon::count(star, &body),
        ..body
    }
}

/// Returns the day in seconds of a body born as `born` on an orbit of `orbit` metres around
/// `star`: its own spin, or its year once the tides of the star have locked it.
pub(super) fn day(star: &StarSpec, orbit: f64, born: &Turning) -> f64 {
    let year = orbit_seconds(star.mu(), orbit);
    spin::settled(born, (star.mu(), orbit, year), star.age)
}

/// Draws the size, the gravity and the spin a solid planet is born with.
///
/// The size is the mean of two draws on the log scale of [`PLANET_RADIUS`], so 7 planets in 10
/// are between 3,000 and 8,000 km. The gravity follows from the size and a drawn rock density,
/// see [`gravity`]. The spin is drawn by [`spin::born`] up to 4 days.
fn drawn(rng: &mut Rng) -> Turning {
    let size = 0.5 * (rng.unit() + rng.unit());
    let radius = PLANET_RADIUS.0 * (PLANET_RADIUS.1 / PLANET_RADIUS.0).powf(size);
    let gravity = gravity(radius, rng.range(ROCK.0, ROCK.1));
    Turning {
        radius,
        gravity,
        day: spin::born(rng.unit(), radius, gravity, SLOWEST),
        quality: QUALITY.0,
    }
}

/// Builds the solid planet number `index` of a star on an orbit of `orbit` metres.
///
/// Its size, gravity and spin are those [`drawn`] for it, and the spin is locked to the orbit
/// where the tides of the star had time to. The record follows from the star, the orbit, the
/// gravity and the size, with its own draws from a child of the seed.
fn solid(star: &StarSpec, index: u8, orbit: f64, seed: u64) -> PlanetSpec {
    let mut rng = Rng::new(seed);
    let born = drawn(&mut rng);
    let [atmosphere_height, cloud_base, cloud_top] = air(born.radius);
    let body = Body::new(born.gravity, born.radius, orbit);
    PlanetSpec {
        id: BodyId {
            star: star.id,
            index,
            moon: 0,
        },
        seed,
        name: names::planet(&star.name, index),
        radius: born.radius,
        gravity: born.gravity,
        day_seconds: day(star, orbit, &born),
        orbit_radius: orbit,
        orbit_seconds: orbit_seconds(star.mu(), orbit),
        orbit_phase: rng.range(0.0, TAU),
        atmosphere_height,
        cloud_base,
        cloud_top,
        record: PlanetRecord::derive(star, &body, hash::child(seed, RECORD_STREAM)),
        host: None,
        moons: 0,
        rings: None,
    }
}
