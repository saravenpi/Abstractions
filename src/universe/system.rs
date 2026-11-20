//! Systems: the planets of a star, on orbits that never cross.

use super::hash::{self, Rng};
use super::planet;
use super::spec::{PlanetSpec, StarSpec};
use super::units::{AU, SUN_MASS};

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

/// Times a planet is moved out and built again to clear the one inside it. Moving out can turn
/// it into a gas giant once, which widens its Hill radius, so three do.
const PUSHES: usize = 3;

/// Mutual Hill radii that two neighbouring planets keep between their orbits: `2 sqrt(3)`, the
/// least spacing at which two planets on circular orbits stay stable (Gladman 1993, "Dynamics
/// of systems of two close planets", Icarus 106, 247). The mutual Hill radius is
/// `((m1 + m2) / 3 M)^(1/3) (a1 + a2) / 2`. Real systems keep far more, 10 or so, so this is a
/// floor, not a typical gap.
pub const SPACING: f64 = 3.464_101_6;

/// Returns the planets of a star, ordered outward.
///
/// The span of orbit radii is cut into one equal band per planet on a log scale and each planet
/// sits in the middle 70 percent of its band, so the radii grow geometrically, with jitter, and
/// each one is strictly larger than the one before. A planet closer to the one inside it than
/// [`SPACING`] mutual Hill radii is moved out until it clears it, see [`placed`], so a crowded
/// system can reach a little past the span.
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

/// Returns the smallest orbit radius in metres for `outer`, a planet of `star`, that keeps it
/// [`SPACING`] mutual Hill radii outside `inside`.
///
/// With `h = SPACING / 2 * ((m1 + m2) / 3 M)^(1/3)` the condition `a2 - a1 >= h (a1 + a2)` gives
/// `a2 >= a1 (1 + h) / (1 - h)`. The share `h` is held under 0.9, far above any planet.
fn least_orbit(star: &StarSpec, inside: &PlanetSpec, outer: &PlanetSpec) -> f64 {
    let pull = (inside.mu() + outer.mu()) / (3.0 * star.mu());
    let share = (0.5 * SPACING * pull.cbrt()).min(0.9);
    inside.orbit_radius * (1.0 + share) / (1.0 - share)
}

/// Returns planet `index` of `star` with the seed `seed`, on the orbit of `orbit` metres it
/// drew or, when that is closer to the planet `inside` it than [`SPACING`] mutual Hill radii,
/// on the nearest orbit that is not.
fn placed(
    star: &StarSpec,
    inside: Option<&PlanetSpec>,
    (index, orbit, seed): (u8, f64, u64),
) -> PlanetSpec {
    let mut planet = planet::build(star, index, orbit, seed);
    let Some(inside) = inside else {
        return planet;
    };
    for _ in 0..PUSHES {
        let least = least_orbit(star, inside, &planet);
        if planet.orbit_radius >= least {
            break;
        }
        planet = planet::build(star, index, least * (1.0 + 1e-9), seed);
    }
    planet
}
