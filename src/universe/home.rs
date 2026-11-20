//! The home planet: where every new game starts. It is chosen, not made: every planet comes
//! from the same rules, and the home is the nearest one to a Sun-like point of the central
//! galaxy that a robot can land on and that holds life.
//!
//! The point is 0.52 radii out in the plane of the disc, see [`shells::sun_point`]. The centre
//! of a galaxy cannot serve: at real scale it is a void hundreds of light-years wide inside a
//! core packed to the cap, and a search from there finds no neighbourhood like ours.

use bevy::math::{DVec3, IVec3};

use super::galaxy;
use super::ids::{BodyId, GalaxyId, StarId};
use super::record::PlanetClass;
use super::spec::{PlanetSpec, StarSpec};
use super::{LY, SECTOR_LY, shells, system};
use crate::life::{World, odds};

/// Grid cell of the home galaxy, the one cell that always holds a galaxy: the centre of the
/// universe.
pub(super) const HOME_GALAXY: GalaxyId = GalaxyId(IVec3::ZERO);

/// Systems looked at before the search settles for the planet most suited to life.
pub const SEARCH: usize = 300;

/// Thickness in light-years of the shells the search goes through, from the Sun-like point out.
const SHELL: f64 = 10.0;

/// The body a universe without a single solid planet starts on. No seed gets there.
const NOWHERE: BodyId = BodyId {
    star: StarId {
        galaxy: HOME_GALAXY,
        sector: IVec3::ZERO,
        index: 0,
    },
    index: 0,
    moon: 0,
};

/// Returns true when a robot can stand on `planet`: solid ground that is not molten.
pub fn landable(planet: &PlanetSpec) -> bool {
    !matches!(
        planet.record.class,
        PlanetClass::GasGiant | PlanetClass::Lava
    )
}

/// Returns the living planet of `planets`, the planets of `star`, a new game would start on:
/// the landable one whose life climbed highest under the normal odds, the inner one of two
/// alike, or nothing when none lives.
fn living(star: &StarSpec, planets: &[PlanetSpec]) -> Option<BodyId> {
    let alive = |p: &PlanetSpec| {
        let rung = odds::rung(&World::of(p, star), p.seed)?;
        landable(p).then_some((rung as u8, p.id))
    };
    let best = planets.iter().rev().filter_map(alive);
    best.max_by_key(|(rung, _)| *rung).map(|(_, id)| id)
}

/// Returns the landable planet of `planets`, the planets of `star`, most likely to hold life,
/// with that chance, the inner one of two alike.
fn suited(star: &StarSpec, planets: &[PlanetSpec]) -> Option<(f32, BodyId)> {
    let chance = |p: &PlanetSpec| (odds::chance(&World::of(p, star)), p.id);
    let landable = planets.iter().rev().filter(|p| landable(p));
    landable.map(chance).max_by(|a, b| a.0.total_cmp(&b.0))
}

/// What the search has found so far.
#[derive(Default)]
struct Found {
    /// The point the search goes out from, in metres from the galaxy centre.
    from: DVec3,
    /// The nearest living planet and the metres from the search point to its star.
    living: Option<(f64, BodyId)>,
    /// The planet most likely to hold life and that chance.
    spare: Option<(f32, BodyId)>,
    /// How many systems were looked at.
    seen: usize,
}

impl Found {
    /// Looks at the system of `star`, unless a living planet nearer than it is already known.
    fn look(&mut self, star: &StarSpec) {
        let out = star.pos.distance(self.from);
        if self.living.is_some_and(|(nearest, _)| out >= nearest) {
            return;
        }
        let planets = system::planets(star);
        self.seen += 1;
        self.living = living(star, &planets).map(|id| (out, id)).or(self.living);
        let spare = suited(star, &planets);
        if spare.is_some_and(|(chance, _)| self.spare.is_none_or(|(best, _)| chance > best)) {
            self.spare = spare;
        }
    }

    /// Returns true when no shell from `low` light-years out can change the answer: the
    /// nearest living planet is closer than any star left, or enough systems had none.
    fn settled(&self, low: f64) -> bool {
        let slack = 0.5 * 3f64.sqrt() * SECTOR_LY;
        let nearer = |(out, _): (f64, BodyId)| out < (low - slack) * LY;
        self.living.map_or(self.seen >= SEARCH, nearer)
    }
}

/// Returns the home planet of a universe: the nearest planet to the Sun-like point of the home
/// galaxy that is solid, landable and alive under the normal odds of life.
///
/// The search goes through the stars shell by shell from that point. When the nearest
/// [`SEARCH`] systems hold no living planet it takes the landable planet most likely to hold
/// life among them. The same seed always gives the same home.
pub(super) fn choose(universe: u64) -> BodyId {
    let Some(shape) = galaxy::shape(universe, HOME_GALAXY) else {
        return NOWHERE;
    };
    let from = shells::sun_point(&shape);
    let mut found = Found {
        from,
        ..Found::default()
    };
    let rounds = (2.0 * shape.radius / SHELL).ceil() as u32;
    for round in 0..rounds {
        let low = f64::from(round) * SHELL;
        if found.settled(low) {
            break;
        }
        let stars = shells::shell(&shape, HOME_GALAXY, from, (low, SHELL));
        stars.iter().for_each(|star| found.look(star));
    }
    let chosen = found.living.map(|(_, id)| id);
    chosen.or(found.spare.map(|(_, id)| id)).unwrap_or(NOWHERE)
}
