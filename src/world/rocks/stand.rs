//! The rocks that stand in one loaded chunk, and the props that are left when one is taken.

use bevy::prelude::*;

use super::id::{id_of, is_rock};
use super::tune::SHRINK_STEP;
use super::{Rock, RockId, Taken};
use crate::world::propmesh::stone_reach;
use crate::world::props::extent;
use crate::world::scatter::{Placement, Rig};
use crate::world::trees::Tree;

/// Metres past the body of a rock at which a point still counts as on it.
const SLACK: f32 = 0.2;

/// The big props of a chunk and the rocks and trees among them, kept so that a taken rock or
/// felled tree can leave without the chunk being generated again.
pub struct Stand {
    pub(in crate::world) key: u64,
    pub(in crate::world) props: Vec<Placement>,
    pub(in crate::world) rocks: Vec<Rock>,
    pub(in crate::world) trees: Vec<Tree>,
    pub(in crate::world) changes: Vec<(Vec3, f32)>,
}

/// Returns the rocks of `props`, placed through `rig`, of the chunk with key `key`. A rock without a
/// body has nothing to aim at and is left out.
fn rocks_of(props: &[Placement], key: u64, rig: &Rig, taken: &Taken) -> Vec<Rock> {
    let rocks = props.iter().filter(|p| is_rock(p.plant));
    rocks
        .filter_map(|p| {
            let [radius, height] = extent(p.plant, p.scale);
            let (at, rot) = rig.pose(p);
            (radius > 0.0).then(|| Rock {
                id: id_of(key, p),
                left: taken.left(id_of(key, p)),
                plant: p.plant,
                scale: p.scale,
                at,
                mid: at + rot * Vec3::Y * (height * 0.5),
            })
        })
        .collect()
}

impl Stand {
    /// Returns the stand of the chunk with key `key` whose big props are `props`, placed through
    /// `rig`, already drawn at the size `taken` leaves the rocks. The props are kept only when a rock
    /// stands among them.
    pub fn of(key: u64, props: &[Placement], rig: &Rig, taken: &Taken) -> Self {
        let rocks = rocks_of(props, key, rig, taken);
        let kept: &[Placement] = if rocks.is_empty() { &[] } else { props };
        Self {
            key,
            props: kept.to_vec(),
            rocks,
            trees: Vec::new(),
            changes: Vec::new(),
        }
    }

    /// Returns the rock whose body holds `at`, a point of the chunk frame, the nearest when several do.
    pub fn near(&self, at: Vec3) -> Option<Rock> {
        let held = self.rocks.iter().filter(|r| {
            stone_reach(r.plant).map_or_else(
                || {
                    let [radius, height] = extent(r.plant, r.scale);
                    r.mid.distance(at) <= radius.hypot(height * 0.5) + SLACK
                },
                |reach| r.at.distance(at) <= reach * r.scale + SLACK,
            )
        });
        let by = |a: &&Rock, b: &&Rock| a.mid.distance(at).total_cmp(&b.mid.distance(at));
        held.min_by(by).copied()
    }

    /// Tells the stand that rock `id` has the share `left` of its mass left. The rock shrinks to the
    /// cube root of what is left, once it is [`SHRINK_STEP`] smaller than drawn, and leaves the stand
    /// when nothing is left. Returns false when no such rock stands here.
    pub fn quarry(&mut self, id: RockId, left: f32) -> bool {
        let Some(at) = self.rocks.iter().position(|r| r.id == id) else {
            return false;
        };
        let rock = &mut self.rocks[at];
        let ratio = if left > 0.0 {
            (left / rock.left).cbrt()
        } else {
            0.0
        };
        if ratio > 1.0 - SHRINK_STEP {
            return true;
        }
        let (key, origin) = (self.key, rock.at);
        if ratio <= 0.0 {
            self.rocks.remove(at);
            self.props.retain(|p| id_of(key, p) != id);
        } else {
            rock.scale *= ratio;
            rock.mid = rock.at + (rock.mid - rock.at) * ratio;
            rock.left = left;
            let props = self.props.iter_mut().filter(|p| id_of(key, p) == id);
            props.for_each(|p| p.scale *= ratio);
        }
        self.changes.push((origin, ratio));
        true
    }

    /// Returns true while a rock changed or left the stand and the chunk still draws it as it was.
    pub fn dirty(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Returns the rocks that still stand here.
    pub fn rocks(&self) -> &[Rock] {
        &self.rocks
    }

    /// Returns the big props that are left.
    pub fn props(&self) -> &[Placement] {
        &self.props
    }

    /// Says the chunk was rebuilt, and returns where the rocks that changed stood, each with the ratio
    /// of its new size to its old one: zero for a rock that left.
    pub fn settle(&mut self) -> Vec<(Vec3, f32)> {
        std::mem::take(&mut self.changes)
    }
}
