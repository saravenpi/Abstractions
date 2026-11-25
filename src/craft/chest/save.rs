//! World save persistence for placed chests and torches.

use bevy::prelude::*;

use super::ChestWorld;
use crate::saves::ActiveWorld;

/// Loads saved chests and torches when a local world becomes active.
pub(super) fn load(active: Res<ActiveWorld>, mut world: ResMut<ChestWorld>) {
    if !world.remote {
        world.chests = active.0.save.chests.clone();
    }
}

/// Copies updated chests and torches back into [`ActiveWorld`] for the next save.
pub(super) fn store(world: Res<ChestWorld>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active else {
        return;
    };
    if active.0.save.chests != world.chests {
        active.0.save.chests = world.chests.clone();
    }
}
