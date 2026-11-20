//! What hunts show at one time: the hunts under way, and the carcasses and bones their kills leave.

use bevy::math::DVec3;

use super::{BONES, Carcass, Hunt, MEAT, Outcome};
use crate::life::herd::{Search, arc};
use crate::life::{Guild, Life};

/// Returns the hunts of `hunts` under way, or whose carcass still has meat, at the search time, with
/// the hunters or the kill within the search radius.
pub(in crate::life) fn seen(life: &Life, hunts: &[Hunt], search: &Search) -> Vec<Hunt> {
    let time = search.time;
    hunts
        .iter()
        .filter(|h| h.start <= time && (time < h.end() || time < meat(life, h)))
        .filter(|h| [h.from, h.to].iter().any(|p| within(search, *p)))
        .copied()
        .collect()
}

/// Returns the carcasses and bones the kills of `hunts` leave within the search radius at the search time.
pub(in crate::life) fn remains(life: &Life, hunts: &[Hunt], search: &Search) -> Vec<Carcass> {
    let time = search.time;
    let visitors: Vec<u16> = life
        .species
        .iter()
        .filter(|s| matches!(s.role.guild, Guild::Scavenger | Guild::Decomposer))
        .map(|s| s.id.index)
        .collect();
    hunts
        .iter()
        .filter_map(|h| match h.outcome {
            Outcome::Kill { slot, at } => Some((h, slot, at)),
            Outcome::Escape => None,
        })
        .map(|(h, slot, at)| Carcass {
            herd: h.prey,
            slot,
            species: h.prey_species,
            at,
            since: h.outcome_time(),
            meat: meat(life, h),
            bones: h.outcome_time() + (meat(life, h) - h.outcome_time()) * BONES,
            visitors: visitors.clone(),
        })
        .filter(|c| c.since <= time && time < c.bones && within(search, c.at))
        .collect()
}

/// Returns true when unit direction `dir` is within the search radius.
fn within(search: &Search, dir: DVec3) -> bool {
    arc(dir, search.at) * search.planet <= search.radius
}

/// Returns the universe time the meat of the kill of `hunt` is gone: minutes, longer for heavier prey.
/// An escape leaves none.
fn meat(life: &Life, hunt: &Hunt) -> f64 {
    let Outcome::Kill { .. } = hunt.outcome else {
        return f64::NEG_INFINITY;
    };
    let mass = life
        .species
        .get(usize::from(hunt.prey_species))
        .map_or(1.0, |s| s.history.mass);
    hunt.outcome_time() + (60.0 * f64::from(mass).sqrt()).clamp(120.0, MEAT)
}
