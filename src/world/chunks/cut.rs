//! The draw cut: which coarse nodes and detail chunks are visible, so the globe is drawn once.

use std::collections::HashSet;

use bevy::prelude::*;

use super::Chunks;
use super::dither::Way;
use super::scene::Scene;
use super::sight::Eye;
use crate::world::globe::quadtree::{Tree, children, parent};
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

    /// Hides the nodes that left the cut: one the player can see fades out, the others go at
    /// once.
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
            if self.sees(*id) {
                let at = (Way::Hide, self.now);
                chunks
                    .fades
                    .begin(commands, node.entity, vec![node.entity], at);
            } else {
                chunks.fades.cancel(commands, node.entity);
                node.show(commands);
            }
        }
    }

    /// Shows the nodes that joined the cut. A node fades in only where the node it takes over
    /// from fades out: its parent after a split, or its children after a merge, which started
    /// its fade already. It appears at once where a detail chunk just went, which leaves no
    /// time for a fade.
    fn show_nodes(&self, commands: &mut Commands, chunks: &mut Chunks) {
        let way = |chunks: &Chunks, id: Option<ChunkId>| {
            let node = id.and_then(|id| chunks.nodes.live.get(&id));
            node.and_then(|node| chunks.fades.way(node.entity))
        };
        let hidden = |id: &&ChunkId| {
            let node = chunks.nodes.live.get(*id);
            node.is_some_and(|node| !node.visible) && !chunks.loaded.contains_key(*id)
        };
        for id in self.cut.iter().filter(hidden).copied().collect::<Vec<_>>() {
            let merged = way(chunks, Some(id)) == Some(Way::In);
            let split = way(chunks, parent(id)) == Some(Way::Hide) && self.sees(id);
            let Some(node) = chunks.nodes.live.get_mut(&id) else {
                continue;
            };
            node.visible = true;
            node.show(commands);
            if split {
                let at = (Way::In, self.now);
                chunks
                    .fades
                    .begin(commands, node.entity, vec![node.entity], at);
            } else if !merged {
                chunks.fades.cancel(commands, node.entity);
            }
        }
    }

    /// Shows the detail chunks whose node is on the cut and hides the others. A chunk that
    /// appears where the player can see it fades in, with everything that stands on it.
    fn show_chunks(&self, commands: &mut Commands, chunks: &mut Chunks, scene: &Scene) {
        for (id, loaded) in chunks.loaded.iter_mut() {
            let wanted = self.cut.contains(id);
            if loaded.shown == wanted {
                continue;
            }
            loaded.shown = wanted;
            let visibility = [Visibility::Hidden, Visibility::Inherited][usize::from(wanted)];
            commands.entity(loaded.entity).try_insert(visibility);
            chunks.fades.cancel(commands, loaded.entity);
            if wanted && self.sees(*id) {
                let (parts, at) = (scene.parts(loaded.entity), (Way::In, self.now));
                chunks.fades.begin(commands, loaded.entity, parts, at);
            }
        }
    }
}

/// Shows the nodes of the draw cut and hides the others. A detail chunk replaces the coarse
/// mesh of its node, and stays hidden until its node is on the cut. What changes where the
/// player can see it fades over a moment; the fades run every frame, and the cut is only looked
/// at again when a node or a chunk appears or goes.
pub fn show(mut commands: Commands, mut chunks: ResMut<Chunks>, scene: Scene) {
    let chunks = &mut *chunks;
    chunks.fades.span = scene.span();
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
    swap.show_chunks(&mut commands, chunks, &scene);
}
