//! The home planet: where every new game starts. It is chosen, not made: every planet comes
//! from the same rules, and the home is the nearest one to a Sun-like point of the central
//! galaxy that a robot can land on and that a new player can wake to life on, see [`homely`].
//!
//! The point is 0.52 radii out in the plane of the disc, see [`shells::sun_point`]. The centre
//! of a galaxy cannot serve: at real scale it is a void hundreds of light-years wide inside a
//! core packed to the cap, and a search from there finds no neighbourhood like ours.

use std::f64::consts::TAU;

use bevy::math::{DVec3, IVec3};

use super::daystar::Daystar;
use super::earthlike;
use super::galaxy;
use super::ids::{BodyId, GalaxyId, StarId};
use super::record::{PlanetClass, Water};
use super::spec::{PlanetSpec, StarSpec};
use super::{LY, SECTOR_LY, STANDARD_GRAVITY, shells, system};
use crate::climate::{Fair, Setting};
use crate::life::{Rung, Solvent, World, chem, odds};

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

/// Coldest and warmest mean surface temperature in kelvin of a home: freezing to 35 C, so that
/// some latitude of the planet, or some longitude of a locked one, is walkable all year.
pub const MILD: (f64, f64) = (273.15, 308.15);

/// Height in degrees above the horizon the star must reach over a day where a home base stands.
///
/// At 25 degrees the ground takes 0.42 of the light of a star overhead and the sky is in full day,
/// which begins at 14.5 degrees (a sine of 0.25, the top of `render::daylight`), so the plants around
/// the base grow in real light and shadows are 2.1 times as long as what casts them. On a planet
/// that turns, every latitude under 65 degrees has it. On a planet locked to its star, where the
/// star never moves, it keeps the base 25 degrees inside the day side, with the star well clear
/// of the hills on the horizon.
pub const LIT: f64 = 25.0;

/// Lowest and highest latitude in degrees, north or south, a new game starts at and a home base
/// stands at: mild land, off the poles and the tropics. The shores of the drainage grid are taken
/// from it, and a home must have a fair place in it.
pub const SHORES: (f64, f64) = (20.0, 48.0);

/// Latitudes in each half and longitudes of the grid a planet is looked over for a fair place:
/// every 7 degrees across [`SHORES`], north and south, every 3 degrees round.
const GRID: (usize, usize) = (5, 120);

/// Lowest and highest surface gravity in metres per second squared of a home: 0.7 and 1.3
/// standard gravities.
///
/// It is what a body built for 9.81 m/s2 walks and flies in comfortably. At 0.7 the jump rises
/// 1.59 m (1.11 m at standard), the pack carries 35.7 kg (25 kg at standard), and the vertical
/// thrusters give the ship 1.8 standard gravities of net lift. At 1.3 the jump rises 0.86 m, the
/// pack carries 19.2 kg, and the net lift is 1.2 standard gravities. At 2.4 the jump is 0.46 m,
/// the pack 10.2 kg and the net lift 0.1 standard gravities.
pub const COMFORT: (f64, f64) = (0.7 * STANDARD_GRAVITY, 1.3 * STANDARD_GRAVITY);

/// Returns true when a new player can wake on `planet`, around `star`, and find life to see.
///
/// Every planet gets the same test, on what is true of it: its life climbed to the skeletal
/// rung or higher, where plants stand in layered forests and animals walk in herds, that life
/// is built on water, the planet has seas and land in balance, water is liquid at the surface,
/// the mean temperature is between 0 and 35 C, the surface gravity is within [`COMFORT`], and
/// some place where a base can stand is fair, see [`fair`]. It is the whole rule of a home in
/// 0.11.0; a new world asks more of its air, see [`earthlike::earthlike`].
pub(super) fn homely(planet: &PlanetSpec, rung: Rung, world: &World) -> bool {
    let record = &planet.record;
    let water = chem::solvent(world).is_some_and(|(solvent, _)| solvent == Solvent::Water);
    let mild = (MILD.0..=MILD.1).contains(&record.temperature);
    let seas = record.class == PlanetClass::Island && record.water == Water::Liquid;
    let pull = (COMFORT.0..=COMFORT.1).contains(&planet.gravity);
    rung >= Rung::Skeletal && water && mild && seas && pull && fair(planet)
}

/// Returns true when some place of `planet` at the latitudes a base stands at, [`SHORES`], is fair:
/// over a day the star reaches [`LIT`] there and the annual mean air is mild and under the heat of
/// a desert, see [`Fair::lack`]. The base search ranks its shores by the same rule at the same
/// latitudes. A planet that turns has such a place. A planet locked to its star has none when its
/// day side is a desert wherever the star stands that high, over a mean of about 19 C: its green
/// ground then lies in the dusk or the dark, and no base would see both the day and a plant on it.
fn fair(planet: &PlanetSpec) -> bool {
    let rule = Fair::of(&Setting::of(planet), Daystar::of(planet));
    let (rows, columns) = GRID;
    (0..2 * rows * columns).any(|i| {
        let (row, side) = (i / columns % rows, [1.0, -1.0][i / columns / rows]);
        let degrees = SHORES.0 + (SHORES.1 - SHORES.0) * row as f64 / (rows - 1) as f64;
        let (lat, lon) = (
            side * degrees.to_radians(),
            (i % columns) as f64 * TAU / columns as f64,
        );
        let dir = DVec3::new(lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin());
        rule.lack(dir) == 0.0
    })
}

/// A test a planet whose life climbed to a rung in a world must pass to be a home, such as
/// [`homely`] or [`earthlike::earthlike`].
pub(super) type Fit = fn(&PlanetSpec, Rung, &World) -> bool;

/// Returns the planet of `planets`, the planets of `star`, a new game would start on: the landable
/// one whose life climbed highest under the normal odds, the inner one of two alike, or nothing
/// when none lives. With a `fit` only those that pass it count.
fn living(star: &StarSpec, planets: &[PlanetSpec], fit: Option<Fit>) -> Option<BodyId> {
    let alive = |p: &PlanetSpec| {
        let world = World::of(p, star);
        let rung = odds::rung(&world, p.seed)?;
        let fits = landable(p) && fit.is_none_or(|fit| fit(p, rung, &world));
        fits.then_some((rung as u8, p.id))
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
struct Found {
    /// The point the search goes out from, in metres from the galaxy centre.
    from: DVec3,
    /// The test a home passes.
    fit: Fit,
    /// Systems looked at before the search settles for less than a home.
    reach: usize,
    /// The nearest planet that passes [`Found::fit`] and the metres from the search point to its
    /// star.
    home: Option<(f64, BodyId)>,
    /// The nearest [`homely`] planet and the metres to its star.
    homely: Option<(f64, BodyId)>,
    /// The nearest living planet, homely or not, and the metres to its star.
    living: Option<(f64, BodyId)>,
    /// The planet most likely to hold life and that chance.
    spare: Option<(f32, BodyId)>,
    /// How many systems were looked at.
    seen: usize,
}

impl Found {
    /// Looks at the system of `star`, unless a home nearer than it is already known.
    fn look(&mut self, star: &StarSpec) {
        let out = star.pos.distance(self.from);
        if self.home.is_some_and(|(nearest, _)| out >= nearest) {
            return;
        }
        let planets = system::planets(star);
        self.seen += 1;
        let at = |id: BodyId| (out, id);
        self.home = living(star, &planets, Some(self.fit)).map(at).or(self.home);
        if self.homely.is_none_or(|(nearest, _)| out < nearest) {
            self.homely = living(star, &planets, Some(homely)).map(at).or(self.homely);
        }
        if self.living.is_none() {
            self.living = living(star, &planets, None).map(at);
        }
        let spare = suited(star, &planets);
        if spare.is_some_and(|(chance, _)| self.spare.is_none_or(|(best, _)| chance > best)) {
            self.spare = spare;
        }
    }

    /// Returns true when no shell from `low` light-years out can change the answer: the
    /// nearest home is closer than any star left, or [`Found::reach`] systems had none.
    fn settled(&self, low: f64) -> bool {
        let slack = 0.5 * 3f64.sqrt() * SECTOR_LY;
        let nearer = |(out, _): (f64, BodyId)| out < (low - slack) * LY;
        self.home.map_or(self.seen >= self.reach, nearer)
    }
}

/// Returns the home planet of a universe under the home `rule`, see [`earthlike::rule`]: the
/// nearest planet to the Sun-like point of the home galaxy that is solid, landable, alive under
/// the normal odds of life and passes the test of the rule, [`homely`] or
/// [`earthlike::earthlike`].
///
/// The search goes through the stars shell by shell from that point. When the systems the rule
/// reaches hold no home it takes the nearest homely planet among them, failing that the nearest
/// living one, and failing that the landable planet most likely to hold life. The same seed and
/// rule always give the same home.
pub(super) fn choose(universe: u64, rule: u32) -> BodyId {
    let Some(shape) = galaxy::shape(universe, HOME_GALAXY) else {
        return NOWHERE;
    };
    let (fit, reach) = earthlike::rule(rule);
    let from = shells::sun_point(&shape);
    let mut found = Found {
        from,
        fit,
        reach,
        home: None,
        homely: None,
        living: None,
        spare: None,
        seen: 0,
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
    let chosen = found.home.or(found.homely).or(found.living);
    let chosen = chosen.map(|(_, id)| id).or(found.spare.map(|(_, id)| id));
    chosen.unwrap_or(NOWHERE)
}
