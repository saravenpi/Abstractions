//! The home planet: where every new game starts. It is chosen, not made: every planet comes
//! from the same rules, and the home is the nearest one to the centre of the central galaxy
//! that a robot can land on and that holds life.

use bevy::math::IVec3;

use super::galaxy::{self, Shape};
use super::ids::{BodyId, GalaxyId, StarId};
use super::record::PlanetClass;
use super::spec::{PlanetSpec, StarSpec};
use super::{GLY, SECTOR_GLY, sector, system};
use crate::life::{World, odds};

/// Grid cell of the home galaxy, the one cell that always holds a galaxy: the centre of the
/// universe.
pub(super) const HOME_GALAXY: GalaxyId = GalaxyId(IVec3::ZERO);

/// Systems looked at before the search settles for the planet most suited to life.
pub const SEARCH: usize = 300;

/// Thickness in game light-years of the shells the search goes through, from the centre out.
const SHELL: f64 = 10.0;

/// Most shells the search goes through: past the rim of the largest galaxy.
const SHELLS: u32 = 360;

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

/// Returns the stars of the home galaxy whose sector centre lies from `low` to `low + SHELL`
/// game light-years from the galaxy centre, nearest first.
fn shell(shape: &Shape, low: f64) -> Vec<StarSpec> {
    let reach = ((low + SHELL) / SECTOR_GLY).ceil() as i32 + 1;
    let side = 2 * reach;
    let mut stars = Vec::new();
    for i in 0..side * side * side {
        let at = IVec3::new(i % side, i / side % side, i / (side * side)) - reach;
        let out = (at.as_dvec3() + 0.5).length() * SECTOR_GLY;
        if (low..low + SHELL).contains(&out) {
            sector::fill(shape, HOME_GALAXY, at, &mut stars);
        }
    }
    stars.sort_by(|a, b| a.pos.length_squared().total_cmp(&b.pos.length_squared()));
    stars
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
    /// The nearest living planet and the metres from the galaxy centre to its star.
    living: Option<(f64, BodyId)>,
    /// The planet most likely to hold life and that chance.
    spare: Option<(f32, BodyId)>,
    /// How many systems were looked at.
    seen: usize,
}

impl Found {
    /// Looks at the system of `star`, unless a living planet nearer than it is already known.
    fn look(&mut self, star: &StarSpec) {
        let out = star.pos.length();
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

    /// Returns true when no shell from `low` game light-years out can change the answer: the
    /// nearest living planet is closer than any star left, or enough systems had none.
    fn settled(&self, low: f64) -> bool {
        let slack = 0.5 * 3f64.sqrt() * SECTOR_GLY;
        let nearer = |(out, _): (f64, BodyId)| out < (low - slack) * GLY;
        self.living.map_or(self.seen >= SEARCH, nearer)
    }
}

/// Returns the home planet of a universe: the nearest planet to the centre of the home galaxy
/// that is solid, landable and alive under the normal odds of life.
///
/// The search goes through the stars shell by shell from the centre. When the nearest
/// [`SEARCH`] systems hold no living planet it takes the landable planet most likely to hold
/// life among them. The same seed always gives the same home.
pub(super) fn choose(universe: u64) -> BodyId {
    let Some(shape) = galaxy::shape(universe, HOME_GALAXY) else {
        return NOWHERE;
    };
    let mut found = Found::default();
    for round in 0..SHELLS {
        let low = f64::from(round) * SHELL;
        if found.settled(low) {
            break;
        }
        shell(&shape, low).iter().for_each(|star| found.look(star));
    }
    let chosen = found.living.map(|(_, id)| id);
    chosen.or(found.spare.map(|(_, id)| id)).unwrap_or(NOWHERE)
}
