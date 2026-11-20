//! Signs of life: what animals leave on the ground, and the calls they make.
//!
//! Like herds and hunts, every sign is a pure function of the life of a planet, its herds and universe
//! time, so every player finds the same tracks at the same place. Herds that walked leave footprints
//! along their path for [`tune::TRAIL`] seconds; herds that lingered leave droppings and, for plant
//! eaters, grazed patches; species that tend their young keep nests or burrows at the home of their
//! herd; and kills leave bones once the meat is gone. Calls (see [`calls`]) are pitched by body size.

mod bones;
pub mod calls;
mod dens;
mod marks;
#[cfg(test)]
mod rig;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_calls;
#[cfg(test)]
mod tests_cost;
mod tracks;
pub mod tune;

use bevy::math::{DVec3, Vec2};

use super::herd::{Ground, Search, Watch, arc, east};
use super::{Life, Species};

/// What a sign is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mark {
    /// One footprint.
    Print,
    /// A dropping.
    Dropping,
    /// A patch of cropped plants where a herd grazed.
    Grazed,
    /// A nest of gathered plant matter.
    Nest,
    /// The mouth of a burrow.
    Burrow,
    /// The bones of a kill.
    Bones,
}

/// One sign on the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sign {
    /// What it is.
    pub mark: Mark,
    /// Index of the species that left it.
    pub species: u16,
    /// Unit direction where it lies.
    pub at: DVec3,
    /// Unit tangent at `at` it points along: the way the animal walked, or the length of the bones.
    pub facing: DVec3,
    /// Length in metres.
    pub size: f32,
    /// How fresh it is, 1 new to 0 gone.
    pub fresh: f32,
}

/// Fills `out` with every sign within the search radius at the search time, nearest first, at most
/// [`tune::MOST`]. The `watch` must hold the herds within the search radius plus [`tune::REACH`].
pub fn near(life: &Life, ground: &Ground, watch: &mut Watch, search: &Search, out: &mut Vec<Sign>) {
    out.clear();
    let herds = watch.herds(life, ground, search.time);
    for herd in &herds {
        let Some(species) = life.species.get(usize::from(herd.species)) else {
            continue;
        };
        let sight = Sight {
            ground,
            search,
            species,
        };
        tracks::lay(watch, life, herd.id, &sight, out);
        marks::lay(watch, life, herd.id, &sight, out);
        dens::lay(herd, &sight, out);
    }
    bones::lay(life, &watch.carcasses(life, search.time), search, out);
    let from = search.at;
    out.sort_unstable_by(|a, b| arc(a.at, from).total_cmp(&arc(b.at, from)));
    out.truncate(tune::MOST);
}

/// What a herd's signs are read against: the ground, the search, and the species of the herd.
pub(super) struct Sight<'a> {
    /// The planet as herds read it.
    pub ground: &'a Ground<'a>,
    /// Where and when signs are looked for.
    pub search: &'a Search,
    /// The species of the herd.
    pub species: &'a Species,
}

impl Sight<'_> {
    /// Returns true when unit direction `dir` is within `slack` metres past the search radius.
    pub fn within(&self, dir: DVec3, slack: f64) -> bool {
        arc(dir, self.search.at) * self.search.planet <= self.search.radius + slack
    }

    /// Adds `sign` to `out` when it lies within the search radius.
    pub fn keep(&self, sign: Sign, out: &mut Vec<Sign>) {
        if self.within(sign.at, 0.0) {
            out.push(sign);
        }
    }
}

/// Returns the unit direction `offset` metres east and north of unit direction `dir` on a planet of
/// `radius` metres.
pub(super) fn beside(dir: DVec3, offset: Vec2, radius: f64) -> DVec3 {
    let east = east(dir);
    let north = dir.cross(east);
    (dir + (east * f64::from(offset.x) + north * f64::from(offset.y)) / radius).normalize()
}

/// Returns the unit tangent at `dir` that points `angle` radians north of east.
pub(super) fn toward(dir: DVec3, angle: f64) -> DVec3 {
    let east = east(dir);
    east * angle.cos() + dir.cross(east) * angle.sin()
}
