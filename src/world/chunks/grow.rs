//! Growing the quadtree: splits and merges along the path of the focus, and the coarse mesh of
//! every node.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::dither::Way;
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
    /// Forgets the node `child` of a merged node, with its detail chunk and its tasks. Returns
    /// true when it left a mesh that fades out from `fade` seconds; else it is gone at once.
    fn retire(&mut self, commands: &mut Commands, child: ChunkId, fade: Option<f64>) -> bool {
        self.pending.remove(&child);
        if let Some(loaded) = self.loaded.remove(&child) {
            commands.entity(loaded.entity).try_despawn();
        }
        let Some(node) = self.nodes.take(child) else {
            return false;
        };
        match fade.filter(|_| node.visible) {
            Some(now) => {
                let at = (Way::Leave, now);
                self.fades
                    .begin(commands, node.entity, vec![node.entity], at);
            }
            None => commands.entity(node.entity).try_despawn(),
        }
        fade.is_some() && node.visible
    }

    /// Applies a selection: merged nodes lose their children, with their detail and their tasks.
    /// Where `fade` gives the time, the drawn children fade out from then while their parent
    /// fades in; a merge that takes detail chunks away is never faded, the chunks go at once.
    fn reshape(
        &mut self,
        commands: &mut Commands,
        changes: &Changes,
        fade: &dyn Fn(ChunkId) -> Option<f64>,
    ) {
        for id in &changes.merges {
            let kids = children(*id);
            let plain = kids.iter().any(|kid| self.loaded.contains_key(kid));
            let fade = fade(*id).filter(|_| !plain);
            let left = kids.map(|kid| self.retire(commands, kid, fade));
            let parent = self.nodes.live.get(id).map(|node| node.entity);
            if let Some((now, parent)) = fade.zip(parent).filter(|_| left.contains(&true)) {
                self.fades
                    .begin(commands, parent, vec![parent], (Way::In, now));
            }
        }
        self.tree.apply(changes);
        let changed = u64::from(*changes != Changes::default());
        self.revision += changed;
        self.stamp += changed;
    }
}

/// Returns the metres a focus at `height` is taken down to be as high over the sea level sphere as it is
/// over the ground.
pub(super) fn drop(height: &Height) -> f64 {
    height.altitude - height.clearance.max(0.0)
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
    let down = at.normalize_or(DVec3::Y) * drop(&Height::of(globe, at));
    view.focus -= down;
    Path {
        view,
        ahead: chunks.motion.ahead(at, globe.radius()) - down,
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
    let (eye, now) = (scene.eye(), scene.now());
    let fade = |id| eye.filter(|eye| eye.sees(id)).map(|_| now);
    chunks.reshape(&mut commands, &changes, &fade);
    let mut wanted = chunks.tree.leaves();
    wanted.retain(|id| chunks.nodes.missing(*id));
    wanted.sort_by(|a, b| path.cost(*a).total_cmp(&path.cost(*b)));
    let settled = changes == Changes::default() && wanted.is_empty();
    chunks.grown = settled.then(|| key(&chunks));
    chunks.nodes.start(wanted, &spawner);
}
