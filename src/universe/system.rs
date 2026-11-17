//! Systems: the planets of a star, on orbits that never cross.

use super::hash::{self, Rng};
use super::spec::{PlanetSpec, StarSpec};
use super::{home, planet};

/// Fewest and most planets around a star.
pub const PLANETS: (u8, u8) = (2, 8);

/// Smallest and largest orbit radius of a planet in metres.
pub const ORBIT: (f64, f64) = (5e6, 5e8);

/// Returns the planets of a star, ordered outward.
///
/// The span of orbit radii is cut into one equal band per planet on a log scale and each planet
/// sits in the middle 70 percent of its band, so the radii grow geometrically, with jitter, and
/// each one is strictly larger than the one before. With `world` set this is the home system:
/// it has at least 3 planets and the third is the home planet, whose terrain seed is `world`.
pub(super) fn planets(star: &StarSpec, world: Option<u64>) -> Vec<PlanetSpec> {
    let mut rng = Rng::new(hash::child(star.seed, 2));
    let least = if world.is_some() {
        home::HOME_INDEX + 1
    } else {
        PLANETS.0
    };
    let count = (PLANETS.0 + rng.below(u32::from(PLANETS.1 - PLANETS.0) + 1) as u8).max(least);
    let mut planets = Vec::with_capacity(usize::from(count));
    for index in 0..count {
        let band = (f64::from(index) + rng.range(0.15, 0.85)) / f64::from(count);
        let orbit = ORBIT.0 * (ORBIT.1 / ORBIT.0).powf(band);
        let built = planet::build(star, index, orbit, rng.next());
        planets.push(match world {
            Some(seed) if index == home::HOME_INDEX => home::planet(built, seed),
            _ => built,
        });
    }
    planets
}
