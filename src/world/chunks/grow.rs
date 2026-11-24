//! Growing the quadtree: splits and merges along the path of the focus, and the coarse mesh of
//! every node.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::reach::Reach;
use super::scene::Scene;
use super::sight::Height;
use super::spawn::Spawner;
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::{Changes, Path, View, children, select_along, splits};

/// Distance in metres the focus moves before a settled quadtree is selected again.
const STILL: f64 = 4.0;

/// Distance in metres the point ahead of the focus moves before a settled quadtree is selected
/// again: it only steers what is built far away.
const STILL_AHEAD: f64 = 16.0;

impl Chunks {
    /// Forgets the node `child` of a merged node, with its detail chunk and its tasks.
    fn retire(&mut self, commands: &mut Commands, child: ChunkId) {
        self.pending.remove(&child);
        if let Some(loaded) = self.loaded.remove(&child) {
            commands.entity(loaded.entity).try_despawn();
        }
        if let Some(node) = self.nodes.take(child) {
            commands.entity(node.entity).try_despawn();
        }
    }

    /// Returns true when the quadtree is as a focus at planet position `at` wants it: the last
    /// selection was made for that spot, nothing changed since, and it found nothing to split,
    /// merge or build. The ground under the focus then exists at the level its height needs.
    pub(super) fn grown_at(&self, at: DVec3) -> bool {
        let here = (at / STILL).floor().as_ivec3();
        let fits =
            |((cell, _), stamp, _): ((IVec3, IVec3), u64, i32)| cell == here && stamp == self.stamp;
        self.grown.is_some_and(fits) && self.nodes.idle()
    }

    /// Applies a selection: merged nodes lose their children immediately so the parent shown on
    /// the draw cut in the same frame replaces them with no crossfade overlap.
    fn reshape(&mut self, commands: &mut Commands, changes: &Changes) {
        for id in &changes.merges {
            for kid in children(*id) {
                self.retire(commands, kid);
            }
        }
        self.tree.apply(changes);
        let changed = u64::from(*changes != Changes::default());
        self.revision += changed;
        self.stamp += changed;
    }
}

/// Returns the path the quadtree is selected for: from planet position `at` to the point the
/// focus will reach, with `reach`. The nodes lie at the sea level radius, so the path is measured from
/// there as high over them as the focus is over the ground, which keeps a focus in a basin or on a
/// mountain as near its nodes as one on a plain. New ground only splits down to the leaves within the detail
/// radius the pace of the focus leaves. A deck has no detail radius: its nodes split by their
/// width alone.
fn path_of(chunks: &Chunks, spawner: &Spawner, at: DVec3, reach: &Reach) -> Path {
    let globe = spawner.terrain.globe();
    let mut view = View {
        focus: at,
        radius: globe.radius(),
        deepest: globe.deepest_level(),
        reach: 0.0,
        budget: reach
            .node_cap(globe.deepest_level())
            .saturating_sub(chunks.tree.count())
            / 4,
        split: reach.split(),
    };
    let leaf = view.arc(view.deepest) * f64::from(u8::from(globe.has_ground()));
    view.reach = f64::from(reach.rings + 1) * leaf;
    let down = at.normalize_or(DVec3::Y) * Height::of(globe, at).drop();
    view.focus -= down;
    Path {
        view,
        ahead: chunks.motion.ahead(view.focus, globe.radius()),
        reach: f64::from(chunks.motion.pace.rings(reach.rings) + 1) * leaf,
    }
}

/// Splits and merges the quadtree along the path of the focus, a few nodes per frame, and builds
/// the mesh of every node. A node only splits once its own mesh exists, so a merge shows it at
/// once.
///
/// Once the tree fits the path nothing is selected again until the focus moves by [`STILL`]
/// metres, the point ahead by [`STILL_AHEAD`], or a node appears.
pub fn grow(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    scene: Scene,
    mut spawner: Spawner,
) {
    let Some(at) = scene.at().filter(|_| !chunks.far.active()) else {
        return;
    };
    let reach = spawner.reach();
    let path = path_of(&chunks, &spawner, at, &reach);
    let stretch = chunks.stretch;
    let spawned = chunks
        .nodes
        .collect(&mut commands, &mut spawner, (&path, stretch));
    chunks.stamp += spawned as u64;
    let cell = |p: DVec3, step: f64| (p / step).floor().as_ivec3();
    let place = (cell(at, STILL), cell(path.ahead, STILL_AHEAD));
    let key = |chunks: &Chunks| (place, chunks.stamp, reach.rings);
    if chunks.grown == Some(key(&chunks)) && chunks.nodes.idle() {
        return;
    }
    let ready = |id| chunks.nodes.has(id);
    let most = stretch.count(splits(path.view.deepest));
    let changes = select_along(&chunks.tree, &path, &ready, most);
    chunks.reshape(&mut commands, &changes);
    let mut wanted = chunks.tree.leaves();
    wanted.retain(|id| chunks.nodes.missing(*id));
    wanted.sort_by(|a, b| path.cost(*a).total_cmp(&path.cost(*b)));
    let settled = changes == Changes::default() && wanted.is_empty();
    chunks.grown = settled.then(|| key(&chunks));
    chunks.nodes.start(wanted, &spawner);
}
