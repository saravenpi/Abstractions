//! Whether the streaming around the focus has caught up: what the boot screen waits for.

use bevy::prelude::*;

use super::Chunks;

/// True while every detail chunk within the render distance of the focus exists and no coarse
/// node is being built: the world around the player is complete.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Calm(pub bool);

/// Notes whether the streaming has caught up with the focus.
pub fn watch_calm(chunks: Res<Chunks>, mut calm: ResMut<Calm>) {
    let wanted = Calm(chunks.settled.is_some() && chunks.pending.is_empty() && chunks.nodes.idle());
    calm.set_if_neq(wanted);
}
