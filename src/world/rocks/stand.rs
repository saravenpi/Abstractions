//! The rocks that stand in one loaded chunk, and the props that are left when one is taken.

use bevy::prelude::*;

use super::id::{id_of, is_rock};
use super::{Rock, RockId};
use crate::world::props::extent;
use crate::world::scatter::{Placement, Rig};

/// Metres past the body of a rock at which a point still counts as on it.
const SLACK: f32 = 0.2;

/// The big props of a chunk and the rocks among them, kept so that a taken rock can leave without
/// the chunk being generated again.
pub struct Stand {
    key: u64,
    props: Vec<Placement>,
    rocks: Vec<Rock>,
    gone: Vec<Vec3>,
}

/// Returns the rocks of `props`, placed through `rig`, of the chunk with key `key`. A rock without a
/// body has nothing to aim at and is left out.
fn rocks_of(props: &[Placement], key: u64, rig: &Rig) -> Vec<Rock> {
    let rocks = props.iter().filter(|p| is_rock(p.plant));
    rocks
        .filter_map(|p| {
            let [radius, height] = extent(p.plant, p.scale);
            let (at, turn) = rig.frame(p.pos);
            (radius > 0.0).then(|| Rock {
                id: id_of(key, p),
                plant: p.plant,
                scale: p.scale,
                at,
                mid: at + turn * Vec3::Y * (height * 0.5),
            })
        })
        .collect()
}

impl Stand {
    /// Returns the stand of the chunk with key `key` whose big props are `props`, placed through
    /// `rig`. The props are kept only when a rock stands among them.
    pub fn of(key: u64, props: &[Placement], rig: &Rig) -> Self {
        let rocks = rocks_of(props, key, rig);
        let kept: &[Placement] = if rocks.is_empty() { &[] } else { props };
        Self {
            key,
            props: kept.to_vec(),
            rocks,
            gone: Vec::new(),
        }
    }

    /// Returns the rock whose body holds `at`, a point of the chunk frame, the nearest when several do.
    pub fn near(&self, at: Vec3) -> Option<Rock> {
        let reach = |r: &Rock| {
            let [radius, height] = extent(r.plant, r.scale);
            radius.hypot(height * 0.5) + SLACK
        };
        let held = self.rocks.iter().filter(|r| r.mid.distance(at) <= reach(r));
        let by = |a: &&Rock, b: &&Rock| a.mid.distance(at).total_cmp(&b.mid.distance(at));
        held.min_by(by).copied()
    }

    /// Takes rock `id` out of the stand. Returns false when no such rock stands here.
    pub fn remove(&mut self, id: RockId) -> bool {
        let Some(at) = self.rocks.iter().position(|r| r.id == id) else {
            return false;
        };
        let rock = self.rocks.remove(at);
        self.gone.push(rock.at);
        let key = self.key;
        self.props.retain(|p| id_of(key, p) != id);
        true
    }

    /// Returns true while rocks left the stand and the chunk still draws them.
    pub fn dirty(&self) -> bool {
        !self.gone.is_empty()
    }

    /// Returns the rocks that still stand here.
    pub fn rocks(&self) -> &[Rock] {
        &self.rocks
    }

    /// Returns the big props that are left.
    pub fn props(&self) -> &[Placement] {
        &self.props
    }

    /// Says the chunk was rebuilt without the rocks that left, and returns where they stood.
    pub fn settle(&mut self) -> Vec<Vec3> {
        std::mem::take(&mut self.gone)
    }
}
