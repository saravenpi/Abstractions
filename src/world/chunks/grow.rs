//! Growing the quadtree: splits and merges around the focus, and the coarse mesh of every node.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::reach::Reach;
use super::spawn::Spawner;
use crate::world::focus::Focus;
use crate::world::globe::quadtree::{Changes, View, children, select};

/// Distance in metres the focus moves before a settled quadtree is selected again.
const STILL: f64 = 4.0;

impl Chunks {
    /// Applies a selection: merged nodes lose their children, with their detail and their tasks.
    fn reshape(&mut self, commands: &mut Commands, changes: &Changes) {
        for child in changes.merges.iter().flat_map(|id| children(*id)) {
            self.nodes.remove(commands, child);
            self.pending.remove(&child);
            if let Some(loaded) = self.loaded.remove(&child) {
                commands.entity(loaded.entity).try_despawn();
            }
        }
        self.tree.apply(changes);
        let changed = u64::from(*changes != Changes::default());
        self.revision += changed;
        self.stamp += changed;
    }
}

/// Returns what the quadtree is selected for at planet position `at` with `reach`.
fn view_of(chunks: &Chunks, spawner: &Spawner, at: DVec3, reach: &Reach) -> View {
    let globe = spawner.terrain.globe();
    let mut view = View {
        focus: at,
        radius: globe.radius(),
        deepest: globe.leaf_level(),
        reach: 0.0,
        budget: reach.node_cap().saturating_sub(chunks.tree.count()) / 4,
        split: reach.split(),
    };
    view.reach = f64::from(reach.rings + 1) * view.arc(view.deepest);
    view
}

/// Splits and merges the quadtree around the focus, a few nodes per frame, and builds the mesh
/// of every node. A node only splits once its own mesh exists, so a merge shows it at once.
///
/// Once the tree fits the focus nothing is selected again until the focus moves by
/// [`STILL`] metres or a node appears.
pub fn grow(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    focus: Focus,
    mut spawner: Spawner,
) {
    let Some(at) = focus.planet().filter(|_| !chunks.far.active()) else {
        return;
    };
    let reach = spawner.reach();
    let view = view_of(&chunks, &spawner, at, &reach);
    let spawned = chunks.nodes.collect(&mut commands, &mut spawner, &view);
    chunks.stamp += spawned as u64;
    let key = |chunks: &Chunks| ((at / STILL).floor().as_ivec3(), chunks.stamp, reach.rings);
    if chunks.grown == Some(key(&chunks)) && chunks.nodes.idle() {
        return;
    }
    let changes = select(&chunks.tree, &view, &|id| chunks.nodes.has(id));
    chunks.reshape(&mut commands, &changes);
    let mut wanted = chunks.tree.leaves();
    wanted.retain(|id| chunks.nodes.missing(*id));
    wanted.sort_by(|a, b| view.distance(*a).total_cmp(&view.distance(*b)));
    let settled = changes == Changes::default() && wanted.is_empty();
    chunks.grown = settled.then(|| key(&chunks));
    chunks.nodes.start(wanted, &spawner);
}
