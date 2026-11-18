//! Gas giants: planets that formed beyond the frost line on a core heavy enough to gather gas.

use super::hash::{self, Rng};
use super::home::{HOME_GRAVITY, HOME_STAR_MASS};
use super::planet;
use super::record::PlanetRecord;
use super::spec::{PlanetSpec, StarSpec};

/// Smallest and largest radius of a gas giant in metres, measured at its cloud deck, which
/// stands for its sea level. Solid worlds stay inside `PLANET_RADIUS`.
pub const GIANT_RADIUS: (f64, f64) = (60_000.0, 150_000.0);

/// Weakest and strongest gravity at the cloud deck of a gas giant, in home gravities.
pub const GIANT_GRAVITY: (f64, f64) = (1.5, 3.0);

/// Flux under which water ice stays solid in the disc a planet forms from: the frost line.
///
/// A black body is at 170 K there. Every habitable zone edge of the star table lies at a flux
/// of 0.21 or more, so the frost line is always beyond it.
pub const FROST: f64 = 0.14;

/// Core mass, from 0 to 1 around a star of the home mass, a core must pass to gather gas.
///
/// The drawn core is scaled by the square root of the star mass, so a heavier star, with a
/// heavier disc, grows giants more often: four cores in five pass around a red dwarf, nearly
/// nine in ten around the home star.
const CORE: f64 = 0.12;

/// Child of the planet seed the gas giant draws come from: its core, size and pull.
const GIANT_STREAM: u64 = 2;

/// Returns the three gas giant draws of the planet of seed `seed`, each from 0 to 1.
fn draws(seed: u64) -> [f64; 3] {
    let mut rng = Rng::new(hash::child(seed, GIANT_STREAM));
    [rng.unit(), rng.unit(), rng.unit()]
}

/// Returns true when the planet of seed `seed`, which receives `flux` from `star`, formed into
/// a gas giant: beyond the frost line, on a core heavier than [`CORE`].
pub(super) fn forms(star: &StarSpec, flux: f64, seed: u64) -> bool {
    let core = draws(seed)[0] * (star.mass / HOME_STAR_MASS).sqrt();
    flux < FROST && core > CORE
}

/// Turns a solid planet into a gas giant on the same orbit, with the same day, seed and name.
///
/// Radii are spread evenly on a log scale over [`GIANT_RADIUS`] and half of the gravity
/// follows the size, as for solid planets. The air heights scale with the radius as
/// [`planet::air`] scales them, from the cloud deck. The record is [`PlanetRecord::giant`],
/// drawn from `record_seed`.
pub(super) fn grow(star: &StarSpec, planet: PlanetSpec, record_seed: u64) -> PlanetSpec {
    let [_, size, pull] = draws(planet.seed);
    let radius = GIANT_RADIUS.0 * (GIANT_RADIUS.1 / GIANT_RADIUS.0).powf(size);
    let span = GIANT_GRAVITY.1 - GIANT_GRAVITY.0;
    let gravity = (GIANT_GRAVITY.0 + span * 0.5 * (size + pull)) * HOME_GRAVITY;
    let [atmosphere_height, cloud_base, cloud_top] = planet::air(radius);
    PlanetSpec {
        radius,
        gravity,
        atmosphere_height,
        cloud_base,
        cloud_top,
        record: PlanetRecord::giant(star, planet.orbit_radius, record_seed),
        ..planet
    }
}
