//! Extending [`Stand`] with choppable trees: proximity queries and trunk removal.

use bevy::prelude::*;

use super::{Felled, TREE_CHOPS, Tree, TreeId, id_of, is_tree, trees_of};
use crate::world::props::extent;
use crate::world::rocks::{Rock, RockId, Stand, Taken, id_of as rock_id_of, is_rock};
use crate::world::scatter::{Placement, Rig};

/// Metres beyond the tree trunk collider radius within which an aim point counts.
const TREE_SLACK: f32 = 0.35;

/// Extracts standing rocks of `props` placed via `rig`, matching `Stand::of`.
fn rocks_list(props: &[Placement], key: u64, rig: &Rig, taken: &Taken) -> Vec<Rock> {
    props
        .iter()
        .filter(|p| is_rock(p.plant))
        .filter_map(|p| {
            let [radius, height] = extent(p.plant, p.scale);
            let (at, rot) = rig.pose(p);
            let id: RockId = rock_id_of(key, p);
            (radius > 0.0).then(|| Rock {
                id,
                left: taken.left(id),
                plant: p.plant,
                scale: p.scale,
                at,
                mid: at + rot * Vec3::Y * (height * 0.5),
            })
        })
        .collect()
}

/// Returns shortest squared distance from point `p` to line segment `a..=b`.
fn dist_sq_segment(p: Vec3, a: Vec3, b: Vec3) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_squared();
    if len_sq <= 1e-6 {
        return p.distance_squared(a);
    }
    let t = (p - a).dot(ab) / len_sq;
    let clamped = t.clamp(0.0, 1.0);
    p.distance_squared(a + ab * clamped)
}

impl Stand {
    /// Builds a stand keeping props whenever any rock or choppable tree stands in the chunk.
    pub fn with_trees(
        key: u64,
        props: &[Placement],
        rig: &Rig,
        taken: &Taken,
        felled: &Felled,
    ) -> Self {
        let rocks = rocks_list(props, key, rig, taken);
        let trees = trees_of(props, key, rig, felled);
        let kept: &[Placement] = if rocks.is_empty() && trees.is_empty() {
            &[]
        } else {
            props
        };
        Self {
            key,
            props: kept.to_vec(),
            rocks,
            trees,
            changes: Vec::new(),
        }
    }

    /// Returns standing choppable trees in this chunk.
    pub fn trees(&self) -> &[Tree] {
        &self.trees
    }

    /// Returns the standing tree whose trunk segment contains or is nearest to `at`.
    pub fn near_tree(&self, at: Vec3) -> Option<Tree> {
        self.trees
            .iter()
            .filter_map(|tree| {
                let reach = tree.radius + TREE_SLACK;
                let d2 = dist_sq_segment(at, tree.at, tree.top);
                (d2 <= reach * reach).then_some((d2, *tree))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, tree)| tree)
    }

    /// Applies `power` chop hits to tree `id`, removing it and queueing canopy cleanup once felled.
    pub fn fell(&mut self, id: TreeId, power: u8) -> bool {
        let Some(pos) = self.trees.iter().position(|t| t.id == id) else {
            return false;
        };
        let tree = &mut self.trees[pos];
        tree.hits = tree.hits.saturating_add(power.max(1)).min(TREE_CHOPS);
        if tree.hits < TREE_CHOPS {
            return true;
        }
        let removed = self.trees.remove(pos);
        let key = self.key;
        self.props
            .retain(|p| !is_tree(p.plant) || id_of(key, p) != id);
        self.changes.push((removed.at, 0.0));
        true
    }
}
