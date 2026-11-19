//! Planning hunts: when each hunter herd hunts, which prey herd it goes for and how the hunt ends.
//!
//! A hunter hunts about once per interval: once per home day for a big apex hunter, up to four times
//! for a small one. Hunt `i` starts at a drawn time in the first 60 percent of interval `i` and ends
//! within 40 percent of an interval, so hunts of one herd never overlap.

use bevy::math::DVec3;

use super::prey;
use super::{AMBUSH_REACH, HUNT_REACH, Hunt, Outcome, TERRITORY};
use crate::life::herd::{Roam, Scene, hunts, open};
use crate::life::{Guild, Species};
use crate::universe::hash::{Rng, child};

/// Seconds of one day on the home planet, the unit hunting rates are counted in.
const DAY: f64 = 1200.0;

/// Salt of the hunt seeds of a herd.
const HUNTS: u64 = 0x4855_4E54;

/// Returns the seconds between two hunts of `species`: shorter for small hunters, longer up the chain.
fn interval(species: &Species) -> f64 {
    let mass = f64::from(species.history.mass.max(0.01));
    let level = f64::from(species.role.level.max(1.0));
    let rate = (10.0 * mass.powf(-0.35)).clamp(0.7, 4.0) * (3.0 / level).min(1.0);
    (DAY / rate).max(300.0)
}

/// Returns every hunt in `scene` that starts within `span`, or is under way at its start, ordered by
/// start time, of the hunters that can be seen within `focus` or can hunt a herd seen there.
pub(super) fn all(scene: &Scene, span: (f64, f64), focus: (DVec3, f64)) -> Vec<Hunt> {
    let mut list: Vec<Hunt> = scene
        .roams
        .iter()
        .filter_map(|r| Some((r, scene.species(r)?)))
        .filter(|(r, s)| hunts(s) && !s.prey.is_empty() && matters(scene, r, s, focus))
        .flat_map(|(r, s)| of_herd(scene, r, s, span))
        .collect();
    list.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.hunter.cmp(&b.hunter)));
    list
}

/// Returns true when hunter `roam` of `species` can be seen within `focus`, or one of its prey herds can.
fn matters(scene: &Scene, roam: &Roam, species: &Species, (at, radius): (DVec3, f64)) -> bool {
    let near = |r: &Roam, out: f64| scene.metres(r.home, at) <= radius + r.reach + out + scene.side;
    near(roam, HUNT_REACH + 50.0)
        || scene
            .roams
            .iter()
            .filter(|r| species.prey.contains(&r.species))
            .any(|r| near(r, 0.0) && scene.metres(r.home, roam.home) <= TERRITORY)
}

/// Returns the hunts of `roam` of `species` that start within `span`, or are under way at its start.
fn of_herd(scene: &Scene, roam: &Roam, species: &Species, (from, to): (f64, f64)) -> Vec<Hunt> {
    let every = interval(species);
    let first = ((from - 0.4 * every) / every).floor() as i64 - 1;
    let last = (to / every).floor() as i64;
    (first..=last)
        .filter_map(|i| one(scene, roam, species, i, every))
        .filter(|h| h.end() > from && h.start <= to)
        .collect()
}

/// Returns hunt `index` of `roam` of `species`, hunting every `every` seconds, if it hunts then.
fn one(scene: &Scene, roam: &Roam, species: &Species, index: i64, every: f64) -> Option<Hunt> {
    let mut rng = Rng::new(child(roam.id ^ HUNTS, index as u64));
    let start = (index as f64 + 0.1 + 0.5 * rng.unit()) * every;
    let (active, _) = open(
        species.role.activity,
        (scene.ground.light)(roam.home, start),
    );
    if !active {
        return None;
    }
    let ambush = species.role.guild == Guild::Ambusher;
    let from = scene.pose(roam, start)?.centre;
    let reach = if ambush { AMBUSH_REACH } else { HUNT_REACH };
    let (prey, seen) = prey::target(scene, (roam, species), from, start, reach)?;
    let distance = scene.metres(from, seen);
    let hunt = Hunt {
        hunter: roam.id,
        prey: prey.id,
        hunter_species: species.id.index,
        prey_species: prey.species,
        start,
        stalk: every * rng.range(0.02, 0.05),
        chase: chase(species, distance, ambush, every),
        feed: 0.0,
        back: 0.1 * every,
        outcome: Outcome::Escape,
        from,
        to: seen,
    };
    prey::strike(scene, hunt, (species, prey), &mut rng, (distance, every))
}

/// Returns the seconds of the chase of a hunter of `species` from `distance` metres, hunting every
/// `every` seconds: hunters close half the gap while they stalk, ambushers all of it in one dash.
fn chase(species: &Species, distance: f64, ambush: bool, every: f64) -> f64 {
    let dash = if ambush {
        distance + 2.0
    } else {
        0.5 * distance + 5.0
    };
    let run = f64::from(species.history.speed).max(0.5);
    (dash / (0.8 * run)).clamp(3.0, 0.1 * every)
}
