//! The draw cut: which coarse nodes and detail chunks are visible, so the globe is drawn once.

use std::collections::HashSet;

use bevy::prelude::*;

use super::Chunks;
use super::dither::Way;
use super::scene::Scene;
use super::sight::Eye;
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

/// What a change of the draw cut needs to know to fade what the player can see.
struct Swap<'a> {
    eye: Option<Eye>,
    now: f64,
    cut: &'a HashSet<ChunkId>,
}

impl Swap<'_> {
    /// Returns true when the player can see the place of `id` through the fog.
    fn sees(&self, id: ChunkId) -> bool {
        self.eye.is_some_and(|eye| eye.sees(id))
    }

    /// Hides the nodes that left the cut immediately so no two terrain surfaces overlap.
    fn hide_nodes(&self, commands: &mut Commands, chunks: &mut Chunks) {
        for (id, node) in chunks
            .nodes
            .live
            .iter_mut()
            .filter(|(_, node)| node.visible)
        {
            if self.cut.contains(id) && !chunks.loaded.contains_key(id) {
                continue;
            }
            node.visible = false;
            chunks.fades.cancel(commands, node.entity);
            node.show(commands);
        }
    }

    /// Shows the nodes that joined the cut immediately without terrain crossfading.
    fn show_nodes(&self, commands: &mut Commands, chunks: &mut Chunks) {
        let hidden = |id: &&ChunkId| {
            let node = chunks.nodes.live.get(*id);
            node.is_some_and(|node| !node.visible) && !chunks.loaded.contains_key(*id)
        };
        for id in self.cut.iter().filter(hidden).copied().collect::<Vec<_>>() {
            let Some(node) = chunks.nodes.live.get_mut(&id) else {
                continue;
            };
            node.visible = true;
            chunks.fades.cancel(commands, node.entity);
            node.show(commands);
        }
    }

    /// Shows the detail chunks whose node is on the cut immediately and fades in only their
    /// props and ground cover where visible.
    fn show_chunks(&self, commands: &mut Commands, chunks: &mut Chunks) {
        for (id, loaded) in chunks.loaded.iter_mut() {
            let wanted = self.cut.contains(id);
            if loaded.shown == wanted {
                continue;
            }
            loaded.shown = wanted;
            let visibility = [Visibility::Hidden, Visibility::Inherited][usize::from(wanted)];
            commands.entity(loaded.entity).try_insert(visibility);
            chunks.fades.cancel(commands, loaded.entity);
            let grass = loaded
                .detail
                .turf
                .grown()
                .flatten()
                .filter(|_| loaded.detail.turf.drawn());
            let cover: Vec<Entity> = [loaded.props, grass].into_iter().flatten().collect();
            if wanted && self.sees(*id) && !cover.is_empty() {
                chunks
                    .fades
                    .begin(commands, loaded.entity, cover, (Way::In, self.now));
            }
        }
    }
}

/// Shows the nodes of the draw cut and hides the others. A detail chunk replaces the coarse
/// mesh of its node immediately, and only cover on the chunk fades in; the fades run every
/// frame, and the cut is only looked at again when a node or a chunk appears or goes.
pub fn show(mut commands: Commands, mut chunks: ResMut<Chunks>, scene: Scene) {
    let chunks = &mut *chunks;
    chunks.fades.span = Some(scene.span());
    chunks.fades.run(&mut commands, scene.now());
    if chunks.drawn == Some(chunks.stamp) {
        return;
    }
    chunks.drawn = Some(chunks.stamp);
    let cut = visible(&chunks.tree, &|id| chunks.nodes.has(id));
    let swap = Swap {
        eye: scene.eye(),
        now: scene.now(),
        cut: &cut,
    };
    swap.hide_nodes(&mut commands, chunks);
    swap.show_nodes(&mut commands, chunks);
    swap.show_chunks(&mut commands, chunks);
}
