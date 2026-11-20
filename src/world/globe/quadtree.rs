//! The chunk quadtree: which nodes of each face are split around the focus.

mod path;
mod pick;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_path;
mod view;

use std::collections::HashSet;

pub use self::path::Path;
#[cfg(test)]
pub use self::pick::select;
pub use self::pick::select_along;
pub use self::view::{View, leaf_at};
use super::{ChunkId, Face};

/// A node splits when the focus is closer than this many times its width.
pub const SPLIT: f64 = 2.0;

/// A node merges when the focus is farther than this many times its width.
pub const MERGE: f64 = 2.4;

/// Most nodes split by one selection on a planet whose tree is [`SHALLOW`] levels deep or less.
pub const MAX_SPLITS: usize = 4;

/// Levels of the tree up to which [`MAX_SPLITS`] splits a selection are enough: a deeper tree
/// needs a split for every extra level or two to reach its leaves in the time a shallow one does.
pub const SHALLOW: u8 = 8;

/// Returns the most nodes split by one selection on a planet whose tree is `depth` levels deep:
/// [`MAX_SPLITS`] up to [`SHALLOW`] levels, then in proportion to the depth, so a 17 level Earth
/// reaches its leaves as fast as a 7 level home planet. Splits only queue work: the mesh of a
/// node is built off the main thread and turned into an entity within the frame budget.
pub fn splits(depth: u8) -> usize {
    (MAX_SPLITS * usize::from(depth))
        .div_ceil(usize::from(SHALLOW))
        .max(MAX_SPLITS)
}

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
    pub(super) fn tips(&self) -> impl Iterator<Item = ChunkId> + '_ {
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
