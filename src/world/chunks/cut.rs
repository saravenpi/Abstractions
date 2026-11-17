//! The draw cut: which coarse nodes and detail chunks are visible, so the globe is drawn once.

use std::collections::HashSet;

use bevy::prelude::*;

use super::Chunks;
use super::raytrace;
use crate::render::RayTracing;
use crate::world::globe::quadtree::{Tree, children};
use crate::world::globe::{ChunkId, Face};

/// Adds to `out` the nodes drawn under `id`: its children when all four have a mesh, else itself.
fn descend(tree: &Tree, has: &dyn Fn(ChunkId) -> bool, id: ChunkId, out: &mut HashSet<ChunkId>) {
    let kids = children(id);
    if tree.is_split(id) && kids.iter().all(|kid| has(*kid)) {
        for kid in kids {
            descend(tree, has, kid, out);
        }
    } else if has(id) {
        out.insert(id);
    }
}

/// Returns the nodes to draw: the deepest nodes of `tree` whose mesh exists, as far down as
/// whole groups of four children are ready. No drawn node covers another.
pub fn visible(tree: &Tree, has: &dyn Fn(ChunkId) -> bool) -> HashSet<ChunkId> {
    let mut out = HashSet::new();
    for face in Face::ALL {
        let root = ChunkId {
            face,
            level: 0,
            x: 0,
            y: 0,
        };
        descend(tree, has, root, &mut out);
    }
    out
}

/// Shows the nodes of the draw cut and hides the others. A detail chunk replaces the coarse
/// mesh of its node, and stays hidden until its node is on the cut. Nothing is looked at again
/// until a node or a chunk appears or goes.
pub fn show(mut commands: Commands, mut chunks: ResMut<Chunks>, ray_tracing: Res<RayTracing>) {
    if chunks.drawn == Some(chunks.stamp) {
        return;
    }
    let chunks = &mut *chunks;
    chunks.drawn = Some(chunks.stamp);
    let cut = visible(&chunks.tree, &|id| chunks.nodes.has(id));
    for (id, node) in chunks.nodes.live.iter_mut() {
        let wanted = cut.contains(id) && !chunks.loaded.contains_key(id);
        if node.visible != wanted {
            node.visible = wanted;
            raytrace::show(&mut commands, node, ray_tracing.0);
        }
    }
    for (id, loaded) in chunks.loaded.iter_mut() {
        let wanted = cut.contains(id);
        if loaded.shown != wanted {
            loaded.shown = wanted;
            let visibility = [Visibility::Hidden, Visibility::Inherited][usize::from(wanted)];
            commands.entity(loaded.entity).try_insert(visibility);
        }
    }
}
