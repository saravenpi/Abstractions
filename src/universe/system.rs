//! Systems: the planets of a star, on orbits that never cross.

use super::hash::{self, Rng};
use super::spec::{PlanetSpec, StarSpec};
use super::units::{AU, SUN_MASS};
use super::{hill, planet};

/// Fewest and most planets around a star.
pub const PLANETS: (u8, u8) = (2, 8);

/// Smallest and largest orbit radius of a planet around a star of the mass of the Sun, in
/// metres: 0.03 to 40 astronomical units. Both scale with the mass of the star, since a lighter
/// star forms its planets from a smaller disc: the planets of a red dwarf of 0.1 Suns orbit
/// from 0.003 to 4 units out.
pub const ORBIT: (f64, f64) = (0.03 * AU, 40.0 * AU);

/// Returns the distance in metres from `star` within which the star shines on its system and the
/// sky draws its planets: four times the widest orbit a planet of this star can have, so the
/// planets pushed out past the span and the dim edge of the system are all inside it.
pub fn reach(star: &StarSpec) -> f64 {
    4.0 * ORBIT.1 * star.mass / SUN_MASS
}

/// Times a planet is moved out and built again to clear the Hill sphere of the one inside it.
/// Moving out can turn it into a gas giant once, which widens its own sphere, so three do.
const PUSHES: usize = 3;

/// Returns the planets of a star, ordered outward.
///
/// The span of orbit radii is cut into one equal band per planet on a log scale and each planet
/// sits in the middle 70 percent of its band, so the radii grow geometrically, with jitter, and
/// each one is strictly larger than the one before. A planet whose Hill sphere would touch the
/// sphere of the planet inside it is moved out until it clears it, see [`placed`], so a
/// crowded system can reach a little past the span.
pub(super) fn planets(star: &StarSpec) -> Vec<PlanetSpec> {
    let mut rng = Rng::new(hash::child(star.seed, 2));
    let count = PLANETS.0 + rng.below(u32::from(PLANETS.1 - PLANETS.0) + 1) as u8;
    let scale = star.mass / SUN_MASS;
    let mut planets: Vec<PlanetSpec> = Vec::with_capacity(usize::from(count));
    for index in 0..count {
        let band = (f64::from(index) + rng.range(0.15, 0.85)) / f64::from(count);
        let orbit = scale * ORBIT.0 * (ORBIT.1 / ORBIT.0).powf(band);
        let planet = placed(star, planets.last(), (index, orbit, rng.next()));
        planets.push(planet);
    }
    planets
}

/// Returns planet `index` of `star` with the seed `seed`, on the orbit of `orbit` metres it
/// drew or, when that is too close to the planet `inside` it, on the nearest orbit where the
/// Hill spheres of the two do not touch.
fn placed(
    star: &StarSpec,
    inside: Option<&PlanetSpec>,
    (index, orbit, seed): (u8, f64, u64),
) -> PlanetSpec {
    let floor = inside.map_or(0.0, |p| p.orbit_radius + hill::hill(star, p));
    let mut planet = planet::build(star, index, orbit, seed);
    for _ in 0..PUSHES {
        let share = hill::hill(star, &planet) / planet.orbit_radius;
        let least = floor / (1.0 - share);
        if planet.orbit_radius > least {
            break;
        }
        planet = planet::build(star, index, least * (1.0 + 1e-9), seed);
    }
    planet
}
