//! Gas giants: planets that formed beyond the frost line on a core heavy enough to gather gas.

use super::air;
use super::hash::{self, Rng};
use super::planet;
use super::record::PlanetRecord;
use super::spec::{GRAVITATION, PlanetSpec, StarSpec};
use super::spin::{self, QUALITY, SLOWEST_GIANT, Turning};
use super::units::SUN_MASS;
use super::volatiles::FROST;

/// Smallest and largest radius of a gas giant in metres, measured at its cloud deck, which
/// stands for its sea level: from Neptune to Jupiter. Solid worlds stay inside `PLANET_RADIUS`.
pub const GIANT_RADIUS: (f64, f64) = (2.5e7, 7.0e7);

/// Mass in kilograms a giant of the smallest and of the largest radius has before its own
/// draw: 15 Earths, a little under Neptune, and the 318 Earths of Jupiter.
pub const GIANT_MASS: (f64, f64) = (9.0e25, 1.9e27);

/// Most a giant is heavier or lighter than its size says. Past the size of Saturn a giant
/// packs more mass into the same radius, so the same size hides masses a factor of 4 apart.
pub const MASS_SPREAD: f64 = 2.0;

/// Core mass, from 0 to 1 around a star of the mass of the Sun or more, a core must pass to
/// gather gas.
///
/// The drawn core is scaled by the mass of the star in Suns, up to 1, so a lighter star, with a
/// lighter disc, grows giants less often: nearly nine cores in ten pass around the Sun, four
/// in ten around a red dwarf of 0.2 Suns and none under 0.12.
const CORE: f64 = 0.12;

/// Child of the planet seed the gas giant draws come from: its core, size, pull and spin.
const GIANT_STREAM: u64 = 2;

/// Returns the four gas giant draws of the planet of seed `seed`, each from 0 to 1.
fn draws(seed: u64) -> [f64; 4] {
    let mut rng = Rng::new(hash::child(seed, GIANT_STREAM));
    [rng.unit(), rng.unit(), rng.unit(), rng.unit()]
}

/// Returns true when the planet of seed `seed`, which receives `flux` from `star`, formed into
/// a gas giant: beyond the frost line, on a core heavier than [`CORE`].
pub(super) fn forms(star: &StarSpec, flux: f64, seed: u64) -> bool {
    let core = draws(seed)[0] * (star.mass / SUN_MASS).min(1.0);
    flux < FROST && core > CORE
}

/// Returns the gravity at the cloud deck of a giant of `radius` metres, in metres per second
/// squared, for a draw `size` that set its radius and a draw `pull`, both from 0 to 1.
///
/// The mass runs on a log scale over [`GIANT_MASS`] with the size, times up to [`MASS_SPREAD`]
/// either way with the pull: 4.8 for a light giant the size of Neptune, 25.9 for a Jupiter and
/// 52 for the heaviest.
fn gravity(radius: f64, size: f64, pull: f64) -> f64 {
    let mass = GIANT_MASS.0 * (GIANT_MASS.1 / GIANT_MASS.0).powf(size);
    GRAVITATION * mass * MASS_SPREAD.powf(2.0 * pull - 1.0) / (radius * radius)
}

/// Turns a solid planet into a gas giant on the same orbit, with the same seed and name.
///
/// Radii are spread evenly on a log scale over [`GIANT_RADIUS`] and the gravity follows the
/// mass, see [`gravity`]. A giant turns fast, see [`SLOWEST_GIANT`], unless the tides of its
/// star locked it. The record is [`PlanetRecord::giant`], drawn from `record_seed`, and the air
/// heights follow from it as [`air::heights`] gives them, from the cloud deck.
pub(super) fn grow(star: &StarSpec, planet: PlanetSpec, record_seed: u64) -> PlanetSpec {
    let [_, size, pull, turn] = draws(planet.seed);
    let radius = GIANT_RADIUS.0 * (GIANT_RADIUS.1 / GIANT_RADIUS.0).powf(size);
    let gravity = gravity(radius, size, pull);
    let born = Turning {
        radius,
        gravity,
        day: spin::born(turn, radius, gravity, SLOWEST_GIANT),
        quality: QUALITY.1,
    };
    let record = PlanetRecord::giant(star, planet.orbit_radius, record_seed);
    let [atmosphere_height, cloud_base, cloud_top] = air::heights(&record, gravity, radius);
    PlanetSpec {
        radius,
        gravity,
        day_seconds: planet::day(star, planet.orbit_radius, &born),
        atmosphere_height,
        cloud_base,
        cloud_top,
        record,
        ..planet
    }
}
