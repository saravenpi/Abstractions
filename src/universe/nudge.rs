//! Fitting a home to the Earth when no planet the search reaches is Earth-like: the nearest
//! homely planet that can be is given an Earth-like pull, air and warmth at generation, and every
//! value that follows from them is derived again from those.

use super::earthlike::{PULL, earthlike};
use super::home::landable;
use super::ids::BodyId;
use super::planet::RECORD_STREAM;
use super::record::{Body, Nudge, PlanetRecord};
use super::spec::{PlanetSpec, StarSpec};
use super::{Universe, air, hash, system};
use crate::life::{World, odds};

/// Sea-level pressures a fitted home may be given, the Earth's first and then further from it,
/// all within [`super::PRESSURE`].
const PRESSURES: [f64; 9] = [1.0, 0.9, 1.1, 0.8, 1.2, 0.7, 1.3, 1.4, 1.5];

/// Carbon a fitted home is given to warm it: a full supply, so its cycle of carbon holds its
/// ground at 280 K at least, as on the early Earth.
const WARM: f64 = 1.0;

/// How a home was fitted to the Earth: which of [`PRESSURES`] it was given and whether it was
/// warmed by a full cycle of carbon. Its pull is held within [`PULL`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fitted {
    /// Index into [`PRESSURES`].
    pressure: u8,
    /// Whether its carbon is [`WARM`] in place of its own.
    warm: bool,
}

/// Returns `planet` of `star` fitted as `fit` says: its surface gravity held within [`PULL`] by
/// its mass, its radius kept, and its record, its air and its clouds derived again from its own
/// draws under the pressure and the carbon of `fit`.
fn fitted(star: &StarSpec, planet: &PlanetSpec, fit: Fitted) -> PlanetSpec {
    let gravity = planet.gravity.clamp(PULL.0, PULL.1);
    let body = Body::new(gravity, planet.radius, planet.orbit_radius);
    let nudge = Nudge {
        pressure: Some(PRESSURES[usize::from(fit.pressure)]),
        carbon: fit.warm.then_some(WARM),
    };
    let seed = hash::child(planet.seed, RECORD_STREAM);
    let record = PlanetRecord::nudged(star, &body, seed, nudge);
    let [atmosphere_height, cloud_base, cloud_top] = air::heights(&record, gravity, planet.radius);
    PlanetSpec {
        gravity,
        record,
        atmosphere_height,
        cloud_base,
        cloud_top,
        ..planet.clone()
    }
}

/// Returns true when `planet` of `star` is a landable Earth-like home under the normal odds.
fn like(star: &StarSpec, planet: &PlanetSpec) -> bool {
    let world = World::of(planet, star);
    let rung = odds::rung(&world, planet.seed);
    landable(planet) && rung.is_some_and(|rung| earthlike(planet, rung, &world))
}

/// Returns the first way to fit `planet` of `star` that makes it Earth-like: every pressure of
/// [`PRESSURES`] with its own carbon, then every one warmed, or nothing when none does.
fn fit(star: &StarSpec, planet: &PlanetSpec) -> Option<Fitted> {
    let mut ways = [false, true].into_iter().flat_map(|warm| {
        (0..PRESSURES.len()).map(move |pressure| Fitted {
            pressure: pressure as u8,
            warm,
        })
    });
    ways.find(|&way| like(star, &fitted(star, planet, way)))
}

/// Returns the first of `candidates`, homely planets with the metres to their star and the star,
/// that can be fitted to the Earth, with how, see [`fit`].
pub(super) fn nearest(candidates: &[(f64, StarSpec, BodyId)]) -> Option<(BodyId, Fitted)> {
    candidates.iter().find_map(|(_, star, id)| {
        let planet = system::planets(star).swap_remove(usize::from(id.index));
        fit(star, &planet).map(|way| (*id, way))
    })
}

impl Universe {
    /// Returns the planets of `star` as this universe has them: as generated, with its home
    /// fitted to the Earth when it was.
    pub(super) fn planets_of(&self, star: &StarSpec) -> Vec<PlanetSpec> {
        let mut planets = system::planets(star);
        let home = self.home;
        let index = usize::from(home.index);
        if let Some(fit) = self.fitted.filter(|_| star.id == home.star)
            && let Some(planet) = planets.get_mut(index)
        {
            *planet = fitted(star, planet, fit);
        }
        planets
    }

    /// Returns true when the home of this universe was fitted to the Earth at generation: no
    /// planet the search reached was Earth-like as it was born.
    pub fn fitted(&self) -> bool {
        self.fitted.is_some()
    }
}
