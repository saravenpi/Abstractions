//! The relations rows of a card: what a species eats, what eats it, and its other ties. Only
//! seen species are named; the others are counted as unknown.

use super::card::Row;
use super::seen::Seen;
use crate::life::{Build, Life, Relation};

/// Most relation rows on a card, so that it fits the smallest window.
pub const MOST: usize = 3;

/// Most species named in one row before the rest are counted.
const NAMED: usize = 2;

/// Returns the species at `indices` in words: up to [`NAMED`] seen names, then how many more.
fn names(life: &Life, seen: &Seen, indices: &[u16]) -> String {
    let all = indices
        .iter()
        .filter_map(|i| life.species.get(usize::from(*i)));
    let known: Vec<&str> = all
        .clone()
        .filter(|s| seen.has(s.id))
        .map(|s| s.name.as_str())
        .collect();
    let unknown = all.count() - known.len();
    let shown = known.len().min(NAMED);
    let rest = known.len() - shown + unknown;
    match (shown, rest) {
        (0, n) => format!("{n} unknown"),
        (_, 0) => known.join(" and "),
        (_, n) => format!("{} and {n} more", known[..shown].join(", ")),
    }
}

/// Returns the plain name of what a species builds.
fn build(build: Build) -> &'static str {
    match build {
        Build::Mound => "Mounds",
        Build::Nest => "Nests",
        Build::Hive => "Hives",
        Build::Burrow => "Burrows",
        Build::Web => "Webs",
        Build::Reef => "Reefs",
    }
}

/// Returns the label and the other species of `relation` seen from species `me`, or nothing
/// when it does not involve `me` or is a build.
fn tie(relation: &Relation, me: u16) -> Option<(&'static str, u16)> {
    match *relation {
        Relation::Pollinates { animal, plant } if animal == me => Some(("Pollinates", plant)),
        Relation::Pollinates { animal, plant } if plant == me => Some(("Pollinated by", animal)),
        Relation::Sows { animal, plant } if animal == me => Some(("Sows", plant)),
        Relation::Sows { animal, plant } if plant == me => Some(("Sown by", animal)),
        Relation::Rides { rider, host } if rider == me => Some(("Rides", host)),
        Relation::Rides { rider, host } if host == me => Some(("Carries", rider)),
        Relation::Mimics { mimic, model } if mimic == me => Some(("Looks like", model)),
        Relation::Mimics { mimic, model } if model == me => Some(("Copied by", mimic)),
        _ => None,
    }
}

/// Returns the rows of the ties of species `me` other than eating, grouped by label.
fn ties(life: &Life, seen: &Seen, me: u16) -> Vec<Row> {
    let mut groups: Vec<(&'static str, Vec<u16>)> = Vec::new();
    for (label, other) in life.relations.iter().filter_map(|r| tie(r, me)) {
        match groups.iter_mut().find(|(l, _)| *l == label) {
            Some((_, list)) => list.push(other),
            None => groups.push((label, vec![other])),
        }
    }
    let builds = life.relations.iter().filter_map(|r| match *r {
        Relation::Builds { species, build: b } if species == me => Some(build(b)),
        _ => None,
    });
    let mut rows: Vec<Row> = groups
        .into_iter()
        .map(|(label, list)| Row::Fact(label, names(life, seen, &list)))
        .collect();
    rows.extend(builds.map(|b| Row::Fact("Builds", b.to_string())));
    rows
}

/// Returns the relations rows of species `me` of `life`, at most [`MOST`] of them.
pub fn rows(life: &Life, seen: &Seen, me: u16) -> Vec<Row> {
    let Some(species) = life.species.get(usize::from(me)) else {
        return Vec::new();
    };
    let eaters: Vec<u16> = life
        .species
        .iter()
        .filter(|s| s.prey.contains(&me))
        .map(|s| s.id.index)
        .collect();
    let eats =
        (!species.prey.is_empty()).then(|| Row::Fact("Eats", names(life, seen, &species.prey)));
    let eaten = (!eaters.is_empty()).then(|| Row::Fact("Eaten by", names(life, seen, &eaters)));
    let mut out: Vec<Row> = eats.into_iter().chain(eaten).collect();
    out.extend(ties(life, seen, me));
    out.truncate(MOST);
    out
}
