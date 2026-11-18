//! Selections: the splits and merges that bring the quadtree closer to what a view wants.

#[cfg(test)]
use super::MAX_SPLITS;
use super::path::Path;
#[cfg(test)]
use super::view::View;
use super::{Changes, Tree, parent};
use crate::world::globe::ChunkId;

/// Returns the splits and merges that bring `tree` closer to what `path` wants: only leaves
/// accepted by `ready` split, the lowest `rank` first, at most `most` or the budget of the view.
fn choose(
    tree: &Tree,
    path: &Path,
    ready: &dyn Fn(ChunkId) -> bool,
    most: usize,
    rank: &dyn Fn(ChunkId) -> f64,
) -> Changes {
    let merges: Vec<ChunkId> = tree.tips().filter(|id| path.merges(*id)).collect();
    let kept = |id: &ChunkId| parent(*id).is_none_or(|p| !merges.contains(&p));
    let mut near: Vec<(f64, ChunkId)> = tree
        .leaves()
        .into_iter()
        .filter(|id| path.splits(*id) && kept(id) && ready(*id))
        .map(|id| (rank(id), id))
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let budget = path.view.budget.min(most);
    Changes {
        splits: near.into_iter().take(budget).map(|(_, id)| id).collect(),
        merges,
    }
}

/// Returns the splits and merges that bring `tree` closer to what `view` wants.
///
/// Only leaves accepted by `ready` split, nearest first and at most `MAX_SPLITS` or the budget of
/// the view. A node merges once the focus is `MERGE / SPLIT` times farther than where it splits,
/// so a focus that hovers around a boundary changes nothing.
#[cfg(test)]
pub fn select(tree: &Tree, view: &View, ready: &dyn Fn(ChunkId) -> bool) -> Changes {
    let rank = |id: ChunkId| view.distance(id);
    choose(tree, &Path::still(*view), ready, MAX_SPLITS, &rank)
}

/// Returns the splits and merges that bring `tree` closer to what a focus moving along `path`
/// wants: the ground around the whole stretch to the point ahead, the cheapest node first and at
/// most `most` splits or the budget of the view.
pub fn select_along(
    tree: &Tree,
    path: &Path,
    ready: &dyn Fn(ChunkId) -> bool,
    most: usize,
) -> Changes {
    choose(tree, path, ready, most, &|id| path.cost(id))
}
