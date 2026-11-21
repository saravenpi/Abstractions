//! When the world is ready for play: the ground under the focus exists at the level its height
//! needs. On foot that is the solid chunks under the body. In the air it is the coarse ground
//! the quadtree settles on, so a body that starts high up never waits for chunks that will not
//! come.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Chunks;
use super::near::Around;
use crate::world::focus::Focus;
use crate::world::globe::ChunkId;
use crate::world::globe::quadtree::parent;
use crate::world::{Terrain, WorldReady};

impl Chunks {
    /// Returns true when the detail chunk of leaf `id` exists with its colliders.
    fn solid(&self, id: ChunkId) -> bool {
        self.loaded.get(&id).is_some_and(|loaded| loaded.solid)
    }

    /// Returns true when the quadtree holds `id` as a leaf, so a detail chunk is built for it.
    /// A leaf the tree did not split down to gets none: the focus is too high over it.
    fn awaited(&self, id: ChunkId) -> bool {
        !self.tree.is_split(id) && parent(id).is_none_or(|up| self.tree.is_split(up))
    }

    /// Returns true when a body can start at planet position `at`, whose leaves are `around`:
    /// every leaf under it is solid, or the baked planet stands in for the ground of a focus in
    /// space, or the quadtree has settled for the spot and every leaf it still builds a detail
    /// chunk for is solid. The last one is a focus in the air.
    fn stands(&self, around: &Around, at: DVec3) -> bool {
        if around.near.iter().all(|id| self.solid(*id)) || self.far.active() {
            return true;
        }
        let waits = |id: &ChunkId| self.awaited(*id) && !self.solid(*id);
        self.grown_at(at) && !around.near.iter().any(waits)
    }
}

/// Flags the world ready once the ground under the focus exists at the level its height needs: on
/// foot the chunk under the focus and the chunks that touch it, with colliders; in the air the
/// coarse ground the quadtree settles on. A planet without ground has nothing that must be solid:
/// it is ready at once.
pub fn mark_ready(
    chunks: Res<Chunks>,
    focus: Focus,
    terrain: Res<Terrain>,
    mut ready: ResMut<WorldReady>,
) {
    if ready.0 {
        return;
    }
    if !terrain.globe().has_ground() {
        ready.0 = true;
        return;
    }
    let Some(at) = focus.planet() else {
        return;
    };
    let around = Around::new(&terrain, at);
    if around.is_some_and(|around| chunks.stands(&around, at)) {
        ready.0 = true;
    }
}
