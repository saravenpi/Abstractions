//! Planets: the physical values of one planet, each inside a documented range.

use std::f64::consts::TAU;

use super::hash::Rng;
use super::home::{HOME_GRAVITY, HOME_RADIUS};
use super::ids::BodyId;
use super::names;
use super::spec::{PlanetSpec, StarSpec};

/// Smallest and largest sea level radius of a planet in metres.
pub const PLANET_RADIUS: (f64, f64) = (3000.0, 40_000.0);

/// Weakest and strongest surface gravity of a planet in Earth gravities.
///
/// The home planet stands for 1 Earth gravity, so a planet of `g` Earth gravities pulls with
/// `g * HOME_GRAVITY` metres per second squared: [`HOME_GRAVITY`] plays the part of 9.81.
pub const PLANET_GRAVITY: (f64, f64) = (0.3, 1.8);

/// Shortest and longest day of a planet in seconds.
pub const PLANET_DAY: (f64, f64) = (600.0, 3600.0);

/// Atmosphere top, cloud base and cloud top of the home planet, in metres above sea level.
const HOME_AIR: [f64; 3] = [4000.0, 900.0, 1500.0];

/// Returns the atmosphere top, cloud base and cloud top of a planet in metres above sea level.
///
/// The three heights are the home planet's 4,000, 900 and 1,500 m times the square root of the
/// radius over the home radius, so they run from 0.71 times those on the smallest planet to
/// 2.58 times on the largest.
pub fn air(radius: f64) -> [f64; 3] {
    let scale = (radius / HOME_RADIUS).sqrt();
    HOME_AIR.map(|height| height * scale)
}

/// Returns the seconds a circular orbit of `radius` metres takes around a body of parameter `mu`.
pub fn orbit_seconds(mu: f64, radius: f64) -> f64 {
    TAU * (radius * radius * radius / mu).sqrt()
}

/// Builds planet number `index` of a star on an orbit of `orbit` metres.
///
/// Radii are spread evenly on a log scale, so small planets are the common ones, and half of
/// the gravity follows the size.
pub(super) fn build(star: &StarSpec, index: u8, orbit: f64, seed: u64) -> PlanetSpec {
    let mut rng = Rng::new(seed);
    let size = rng.unit();
    let radius = PLANET_RADIUS.0 * (PLANET_RADIUS.1 / PLANET_RADIUS.0).powf(size);
    let pull = 0.5 * (size + rng.unit());
    let earths = PLANET_GRAVITY.0 + (PLANET_GRAVITY.1 - PLANET_GRAVITY.0) * pull;
    let [atmosphere_height, cloud_base, cloud_top] = air(radius);
    PlanetSpec {
        id: BodyId {
            star: star.id,
            index,
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
    }
}
