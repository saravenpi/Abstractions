//! Hunts and kills: when hunter herds hunt, whom, how it ends, and the carcasses kills leave.
//!
//! Like herds, hunts are pure functions of the life of a planet and universe time, so every player
//! sees the same chase and the same kill. Each hunter herd hunts at times drawn from
//! `hash(herd id, hunt index)`, only in its active hours, and goes for the nearest prey herd of its
//! `prey` list in reach. A kill takes one member of the prey herd for [`RECOVERY`] seconds and leaves
//! a carcass, then bones, that scavengers and decomposers come to.

mod chase;
mod plan;
mod prey;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_motion;

use bevy::math::DVec3;

use super::herd::{DEATHS, Death, Ground, Pose, Scene, Search};
use super::{Guild, Life};

/// Seconds a killed member stays missing from its herd before the herd has grown it back.
pub const RECOVERY: f64 = 1800.0;

/// Farthest in metres a hunter sees prey from.
pub(super) const HUNT_REACH: f64 = 200.0;

/// Farthest in metres between the home of a hunter herd and the homes of the herds it hunts; hunters
/// roam no farther than this from home.
pub(super) const TERRITORY: f64 = 400.0;

/// Farthest in metres an ambusher strikes from.
const AMBUSH_REACH: f64 = 50.0;

/// Most seconds the meat of a carcass lasts.
const MEAT: f64 = 900.0;

/// How many times longer bones last than meat.
const BONES: f64 = 3.0;

/// How a hunt ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// The member in `slot` of the prey herd dies at unit direction `at`.
    Kill {
        /// Slot of the killed member.
        slot: u8,
        /// Unit direction where it dies.
        at: DVec3,
    },
    /// The prey gets away.
    Escape,
}

/// One hunt: a hunter herd stalks, then chases a prey herd, then feeds or gives up, then goes back to
/// its route.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hunt {
    /// Identity of the hunter herd.
    pub hunter: u64,
    /// Identity of the prey herd.
    pub prey: u64,
    /// Species index of the hunter.
    pub hunter_species: u16,
    /// Species index of the prey.
    pub prey_species: u16,
    /// Universe time in seconds the stalk starts.
    pub start: f64,
    /// Seconds of the stalk.
    pub stalk: f64,
    /// Seconds of the chase; it ends with the outcome.
    pub chase: f64,
    /// Seconds the hunters eat the kill, 0 on an escape.
    pub feed: f64,
    /// Seconds the hunters take to rejoin their route.
    pub back: f64,
    /// How it ends.
    pub outcome: Outcome,
    /// Unit direction of the hunter herd when the stalk starts.
    pub from: DVec3,
    /// Unit direction where the chase ends: the kill, or where the prey was when it got away.
    pub to: DVec3,
}

impl Hunt {
    /// Returns the universe time of the outcome.
    pub fn outcome_time(&self) -> f64 {
        self.start + self.stalk + self.chase
    }

    /// Returns the universe time the hunters are back on their route.
    pub fn end(&self) -> f64 {
        self.outcome_time() + self.feed + self.back
    }
}

/// What a kill leaves.
#[derive(Debug, Clone, PartialEq)]
pub struct Carcass {
    /// Identity of the herd the dead member came from.
    pub herd: u64,
    /// Slot of the dead member.
    pub slot: u8,
    /// Species index of the dead member.
    pub species: u16,
    /// Unit direction where it lies.
    pub at: DVec3,
    /// Universe time of the death.
    pub since: f64,
    /// Universe time the meat is gone; bones are left until `bones`.
    pub meat: f64,
    /// Universe time the bones are gone.
    pub bones: f64,
    /// Indices of the scavenger and decomposer species drawn to it.
    pub visitors: Vec<u16>,
}

/// Returns the hunts that are under way, or whose carcass still has meat, at the search time, with
/// the hunters or the kill within the search radius, in a stable order.
pub fn hunts_near(life: &Life, ground: &Ground, search: &Search) -> Vec<Hunt> {
    let scene = Scene::build(life, ground, search);
    let time = search.time;
    plan(&scene, (time - MEAT, time), (search.at, search.radius))
        .into_iter()
        .filter(|h| h.start <= time && (time < h.end() || time < meat(life, h)))
        .filter(|h| {
            [h.from, h.to]
                .iter()
                .any(|p| scene.metres(*p, search.at) <= search.radius)
        })
        .collect()
}

/// Returns the carcasses and bones within the search radius at the search time, in a stable order.
pub fn carcasses_near(life: &Life, ground: &Ground, search: &Search) -> Vec<Carcass> {
    let scene = Scene::build(life, ground, search);
    let time = search.time;
    let visitors: Vec<u16> = life
        .species
        .iter()
        .filter(|s| matches!(s.role.guild, Guild::Scavenger | Guild::Decomposer))
        .map(|s| s.id.index)
        .collect();
    plan(
        &scene,
        (time - MEAT * BONES, time),
        (search.at, search.radius),
    )
    .into_iter()
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
        meat: meat(life, &h),
        bones: h.outcome_time() + (meat(life, &h) - h.outcome_time()) * BONES,
        visitors: visitors.clone(),
    })
    .filter(|c| c.since <= time && time < c.bones && scene.metres(c.at, search.at) <= search.radius)
    .collect()
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

/// Returns every hunt that starts within `span`, or is under way at its start, by start time, of the
/// hunters of `scene` that can be seen within `focus`, a unit direction and a radius in metres, or can
/// hunt a herd that can be seen there.
pub(super) fn plan(scene: &Scene, span: (f64, f64), focus: (DVec3, f64)) -> Vec<Hunt> {
    plan::all(scene, span, focus)
}

/// Returns the pose of a hunter herd whose route gives `base` at `time`, bent by its `hunt`.
pub(super) fn bend(scene: &Scene, base: &Pose, hunt: &Hunt, time: f64) -> Pose {
    chase::bend(scene, base, hunt, time)
}

/// Returns the deaths of herd `id` from `hunts` that have started by `time` and whose member is not
/// yet grown back, the latest [`DEATHS`] of them, earliest first.
pub(super) fn deaths(hunts: &[Hunt], id: u64, time: f64) -> [Option<Death>; DEATHS] {
    let kills: Vec<Death> = hunts
        .iter()
        .filter(|h| h.prey == id && h.start <= time && time < h.outcome_time() + RECOVERY)
        .filter_map(|h| match h.outcome {
            Outcome::Kill { slot, .. } => Some(Death {
                slot,
                at: h.outcome_time(),
            }),
            Outcome::Escape => None,
        })
        .collect();
    let mut deaths = [None; DEATHS];
    let skip = kills.len().saturating_sub(DEATHS);
    for (slot, death) in deaths.iter_mut().zip(kills.into_iter().skip(skip)) {
        *slot = Some(death);
    }
    deaths
}
