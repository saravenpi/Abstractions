//! Rolling herds: how many herds of each species a cell holds, and the herd pinned near the robot.

use bevy::math::{DVec3, IVec3};

use super::cells::Grid;
use crate::life::census::{self, Site};
use crate::life::{Guild, Habitat, Life, Species};
use crate::universe::hash::{Rng, cell, child, mix};

/// Farthest a herd roams from its home in metres, whatever its home range.
pub(super) const MAX_ROAM: f64 = 800.0;

/// Farthest the pinned herd roams from the pin in metres.
const PIN_ROAM: f64 = 250.0;

/// Salt of the herd seeds of a planet.
const HERDS: u64 = 0x4E52_4448;

/// Salt of the pinned herd.
const PINNED: u64 = 0x5049_4E4E;

/// A herd before time is applied: who it is and where it lives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Roam {
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

/// Returns the farthest any species of `life` roams, in metres.
pub(super) fn widest(life: &Life) -> f64 {
    life.species
        .iter()
        .filter(|s| s.role.guild != Guild::Producer)
        .map(|s| f64::from(s.history.range).min(MAX_ROAM))
        .fold(0.0, f64::max)
}

/// Returns the herds that live in cell `at` of `grid`, whose centre has `site`, in a stable order.
pub(super) fn in_cell(life: &Life, grid: &Grid, at: IVec3, site: &Site) -> Vec<Roam> {
    let planet = life.species.first().map_or(0, |s| s.id.planet);
    let seed = cell(child(planet, HERDS), at);
    let centre = grid.spot(at, 0.5, 0.5);
    let mut herds = Vec::new();
    for species in &life.species {
        let seed = child(seed, u64::from(species.id.index));
        let expected = census::herds(species, site) * grid.area();
        let n = (f64::from(expected) + Rng::new(seed).unit()).floor() as u64;
        herds.extend((1..=n).map(|k| {
            let mut rng = Rng::new(child(seed, k));
            let home = grid.spot(at, rng.range(0.1, 0.9), rng.range(0.1, 0.9));
            roam(
                species,
                child(seed, k),
                home,
                centre,
                f64::from(species.history.range).min(MAX_ROAM),
            )
        }));
    }
    herds
}

/// Returns the herd of the most common ground grazer of `life` that lives at `pin`, if it has one.
///
/// The most common is the lightest, which Damuth's law makes the most numerous.
pub(super) fn pinned(life: &Life, pin: DVec3) -> Option<Roam> {
    let species = life
        .species
        .iter()
        .filter(|s| s.role.guild == Guild::Grazer && s.role.habitat == Habitat::Ground)
        .min_by(|a, b| a.history.mass.total_cmp(&b.history.mass))?;
    let bits = [pin.x, pin.y, pin.z]
        .iter()
        .fold(PINNED, |h, v| mix(h ^ v.to_bits()));
    let seed = child(child(species.id.planet, bits), u64::from(species.id.index));
    Some(roam(
        species,
        seed,
        pin,
        pin,
        PIN_ROAM.min(f64::from(species.history.range)),
    ))
}

/// Returns the herd of `species` with `seed`, living at `home`, falling back to `fallback`, within `reach`.
fn roam(species: &Species, seed: u64, home: DVec3, fallback: DVec3, reach: f64) -> Roam {
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
