//! Whether the streaming around the focus has caught up: what the boot screen waits for.

use bevy::prelude::*;

use super::Chunks;
use crate::world::Terrain;

/// True while every detail chunk within the render distance of the focus exists and no coarse
/// node is being built: the world around the player is complete.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Calm(pub bool);

/// Notes whether the streaming has caught up with the focus. A planet without ground has no detail
/// chunk to wait for: it has caught up once its quadtree is grown.
pub fn watch_calm(chunks: Res<Chunks>, terrain: Res<Terrain>, mut calm: ResMut<Calm>) {
    let deck = !terrain.globe().has_ground() && chunks.grown.is_some();
    let detail = chunks.settled.is_some() || deck;
    let wanted = Calm(detail && chunks.pending.is_empty() && chunks.nodes.idle());
    calm.set_if_neq(wanted);
}
