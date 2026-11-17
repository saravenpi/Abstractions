//! The chunk quadtree: which nodes of each face are split around the focus.

#[cfg(test)]
mod tests;
mod view;

use std::collections::HashSet;

pub use self::view::{View, leaf_at};
use super::{ChunkId, Face};

/// A node splits when the focus is closer than this many times its width.
pub const SPLIT: f64 = 2.0;

/// A node merges when the focus is farther than this many times its width.
pub const MERGE: f64 = 2.4;

/// Most nodes split by one selection.
pub const MAX_SPLITS: usize = 4;

/// Most nodes the streamer keeps alive, hidden parents included.
///
/// The home planet at the default render distance settles at about 500 nodes, a 40 km planet at
/// the longest render distance at about 1000.
pub const MAX_NODES: usize = 1200;

/// The quadtrees of the six faces: the set of nodes that are split into four.
///
/// A split node always has a split parent, so the leaves cover the globe exactly once.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tree {
    split: HashSet<ChunkId>,
}

/// What one selection asks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Changes {
    /// Leaves to split into their four children, nearest first.
    pub splits: Vec<ChunkId>,
    /// Split nodes to merge back into one leaf.
    pub merges: Vec<ChunkId>,
}

/// Returns the four nodes that cover `id` one level down.
pub fn children(id: ChunkId) -> [ChunkId; 4] {
    [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(dx, dy)| ChunkId {
        face: id.face,
        level: id.level + 1,
        x: id.x * 2 + dx,
        y: id.y * 2 + dy,
    })
}

/// Returns the node one level up that holds `id`, or nothing for a whole face.
pub fn parent(id: ChunkId) -> Option<ChunkId> {
    (id.level > 0).then(|| ChunkId {
        face: id.face,
        level: id.level - 1,
        x: id.x / 2,
        y: id.y / 2,
    })
}

impl Tree {
    /// Returns true when `id` is split into its four children.
    pub fn is_split(&self, id: ChunkId) -> bool {
        self.split.contains(&id)
    }

    /// Returns the number of nodes, split ones and leaves together.
    pub fn count(&self) -> usize {
        Face::ALL.len() + self.split.len() * 4
    }

    /// Returns every node that is not split. Together they cover the globe exactly once.
    pub fn leaves(&self) -> Vec<ChunkId> {
        let roots = Face::ALL.map(|face| ChunkId {
            face,
            level: 0,
            x: 0,
            y: 0,
        });
        let below = self.split.iter().flat_map(|id| children(*id));
        let all = roots.into_iter().chain(below);
        all.filter(|id| !self.is_split(*id)).collect()
    }

    /// Returns the split nodes whose four children are all leaves.
    fn tips(&self) -> impl Iterator<Item = ChunkId> + '_ {
        let leafy = |id: &ChunkId| children(*id).iter().all(|c| !self.is_split(*c));
        self.split.iter().copied().filter(leafy)
    }

    /// Splits and merges the nodes of `changes`.
    pub fn apply(&mut self, changes: &Changes) {
        self.split.extend(changes.splits.iter().copied());
        for id in &changes.merges {
            self.split.remove(id);
        }
    }
}

/// Returns the splits and merges that bring `tree` closer to what `view` wants.
///
/// Only leaves accepted by `ready` split, nearest first and at most `MAX_SPLITS` or the budget of
/// the view. A node merges once the focus is `MERGE / SPLIT` times farther than where it splits,
/// so a focus that hovers around a boundary changes nothing.
pub fn select(tree: &Tree, view: &View, ready: &dyn Fn(ChunkId) -> bool) -> Changes {
    let merges: Vec<ChunkId> = tree.tips().filter(|id| view.merges(*id)).collect();
    let kept = |id: &ChunkId| parent(*id).is_none_or(|p| !merges.contains(&p));
    let mut near: Vec<(f64, ChunkId)> = tree
        .leaves()
        .into_iter()
        .filter(|id| view.splits(*id) && kept(id) && ready(*id))
        .map(|id| (view.distance(id), id))
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let budget = view.budget.min(MAX_SPLITS);
    Changes {
        splits: near.into_iter().take(budget).map(|(_, id)| id).collect(),
        merges,
    }
}
