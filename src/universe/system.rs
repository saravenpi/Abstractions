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
/// it has at least 5 planets, the third is the home planet, whose terrain seed is `world`, and
/// the orbits are those of [`home::orbit`], which keep the order and put the home planet where
/// its star gives it a flux of 1. The outermost planet of the home system is always a gas
/// giant, so a new player has a giant and its moons to fly to.
pub(super) fn planets(star: &StarSpec, world: Option<u64>) -> Vec<PlanetSpec> {
    let mut rng = Rng::new(hash::child(star.seed, 2));
    let least = if world.is_some() {
        home::HOME_PLANETS
    } else {
        PLANETS.0
    };
    let count = (PLANETS.0 + rng.below(u32::from(PLANETS.1 - PLANETS.0) + 1) as u8).max(least);
    let mut drawn = [(0.0, 0); PLANETS.1 as usize];
    for (index, slot) in (0..count).zip(&mut drawn) {
        let band = (f64::from(index) + rng.range(0.15, 0.85)) / f64::from(count);
        *slot = (band, rng.next());
    }
    let home = world.map(|world| (world, drawn[usize::from(home::HOME_INDEX)].0, count - 1));
    let members = (0..count).zip(drawn);
    members
        .map(|(index, drawn)| member(star, index, drawn, home))
        .collect()
}

/// Returns planet `index` of `star` from the place `band` it drew in the span of orbits, 0 to 1 on
/// the log scale, and its seed. In the home system `home` gives the world seed, the place the
/// home planet drew and the index of the outermost planet.
fn member(
    star: &StarSpec,
    index: u8,
    (band, seed): (f64, u64),
    home: Option<(u64, f64, u8)>,
) -> PlanetSpec {
    let Some((world, at, last)) = home else {
        let orbit = ORBIT.0 * (ORBIT.1 / ORBIT.0).powf(band);
        return planet::build(star, index, orbit, seed, false);
    };
    let planet = planet::build(star, index, home::orbit(band, at), seed, index == last);
    match index == home::HOME_INDEX {
        true => home::planet(planet, world),
        false => planet,
    }
}
