//! Rolling herds: how many herds of each species a cell holds, from the census at the cell centre.

use bevy::math::{DVec3, IVec3};

use super::cells::Grid;
use super::sphere::arc;
use crate::life::census::{self, Site};
use crate::life::hunt::TERRITORY;
use crate::life::{Guild, Life, Species};
use crate::universe::hash::{Rng, cell, child, mix};

/// Farthest a herd roams from its home in metres, whatever its home range.
pub(in crate::life) const MAX_ROAM: f64 = 800.0;

/// Salt of the herd seeds of a planet.
const HERDS: u64 = 0x4E52_4448;

/// A herd before time is applied: who it is and where it lives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::life) struct Roam {
    /// Stable identity, also the seed of its route.
    pub id: u64,
    /// Index of its species.
    pub species: u16,
    /// Members.
    pub count: u8,
    /// Young among the members.
    pub young: u8,
    /// Centre of its home range, a unit direction.
    pub home: DVec3,
    /// Where it goes when no waypoint suits it: a unit direction known to suit it.
    pub fallback: DVec3,
    /// Radius in metres of the disc around `home` its waypoints fall in.
    pub reach: f64,
}

/// Returns true for the species that hunt: hunters and ambushers.
pub(in crate::life) fn hunts(species: &Species) -> bool {
    matches!(species.role.guild, Guild::Hunter | Guild::Ambusher)
}

/// Returns the herds of `grid` whose cell centre is within `reach` metres of `at`, hunters only when
/// `hunters` is true and plant eaters only otherwise. A cell centre is sampled only when the cell could
/// hold a herd even at the best site.
pub(super) fn around(
    life: &Life,
    grid: &Grid,
    (at, reach): (DVec3, f64),
    sample: &dyn Fn(DVec3) -> Site,
    hunters: bool,
) -> Vec<Roam> {
    let planet = life.species.first().map_or(0, |s| s.id.planet);
    let salt = child(planet, HERDS ^ u64::from(hunters));
    let kinds: Vec<&Species> = life
        .species
        .iter()
        .filter(|s| hunts(s) == hunters)
        .collect();
    grid.around(at, reach)
        .into_iter()
        .filter(|c| arc(grid.spot(*c, 0.5, 0.5), at) * grid.radius <= reach)
        .flat_map(|c| {
            let seed = cell(salt, c);
            let may = kinds
                .iter()
                .any(|s| rolled(seed, s, census::most(s) * grid.area()) > 0);
            let site = may.then(|| sample(grid.spot(c, 0.5, 0.5)));
            site.map(|site| in_cell(&kinds, grid, (c, seed), &site))
                .unwrap_or_default()
        })
        .collect()
}

/// Returns how many herds of `species` cell `seed` holds when `expected` are expected there.
fn rolled(seed: u64, species: &Species, expected: f32) -> u64 {
    (f64::from(expected) + Rng::new(child(seed, u64::from(species.id.index))).unit()).floor() as u64
}

/// Returns the herds of `kinds` in cell `at` with `seed` of `grid`, whose centre has `site`.
fn in_cell(kinds: &[&Species], grid: &Grid, (at, seed): (IVec3, u64), site: &Site) -> Vec<Roam> {
    let centre = grid.spot(at, 0.5, 0.5);
    let mut herds = Vec::new();
    for species in kinds {
        let n = rolled(seed, species, census::herds(species, site) * grid.area());
        let seed = child(seed, u64::from(species.id.index));
        let most = if hunts(species) { TERRITORY } else { MAX_ROAM };
        let reach = f64::from(species.history.range).min(most);
        herds.extend((1..=n).map(|k| {
            let mut rng = Rng::new(child(seed, k));
            let home = grid.spot(at, rng.range(0.1, 0.9), rng.range(0.1, 0.9));
            roam(species, child(seed, k), (home, centre), reach)
        }));
    }
    herds
}

/// Returns the herd of `species` with `seed`, living at `home` and falling back to `fallback`, within
/// `reach`.
fn roam(species: &Species, seed: u64, (home, fallback): (DVec3, DVec3), reach: f64) -> Roam {
    let mut rng = Rng::new(mix(seed));
    let history = &species.history;
    let spread = u32::from(history.group_max.saturating_sub(history.group_min)) + 1;
    let count = history
        .group_min
        .max(1)
        .saturating_add(rng.below(spread) as u8);
    let young = (f64::from(count - 1) * rng.range(0.1, 0.35) * f64::from(history.care.max(0.3)))
        .round() as u8;
    Roam {
        id: seed,
        species: species.id.index,
        count,
        young,
        home,
        fallback,
        reach,
    }
}
