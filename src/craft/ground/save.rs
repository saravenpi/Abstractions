//! Keeping the ground in the world file: read when a world opens, written whenever it changes.

use bevy::prelude::*;

use super::Ground;
use crate::saves::ActiveWorld;

/// Reads the ground of the world that was just made active. A world a server keeps comes with no
/// ground: the server tells it.
pub(super) fn load(active: Res<ActiveWorld>, mut ground: ResMut<Ground>) {
    if !ground.remote {
        ground.pile = active.0.save.ground.clone();
    }
}

/// Writes the ground into the active world whenever it changes, so the next save keeps it.
pub(super) fn store(ground: Res<Ground>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active else {
        return;
    };
    if active.0.save.ground != ground.pile {
        active.0.save.ground = ground.pile.clone();
    }
}
