//! Keeping the pack in the world file, and weighing it under the gravity underfoot.

use bevy::prelude::*;

use super::{Burden, Pack};
use crate::saves::ActiveWorld;
use crate::world::Terrain;

/// Reads the pack of the world that was just made active.
pub(crate) fn load(active: Option<Res<ActiveWorld>>, mut pack: ResMut<Pack>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    *pack = active.0.save.pack.clone();
}

/// Writes the pack into the active world whenever it changes, so the next save keeps it.
pub(crate) fn store(pack: Res<Pack>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| pack.is_changed()) else {
        return;
    };
    if active.0.save.pack != *pack {
        active.0.save.pack = pack.clone();
    }
}

/// Weighs the pack under the gravity of the planet underfoot when either changes.
pub(crate) fn weigh(pack: Res<Pack>, terrain: Option<Res<Terrain>>, mut burden: ResMut<Burden>) {
    let moved = terrain.as_ref().is_some_and(|t| t.is_changed());
    if !pack.is_changed() && !moved {
        return;
    }
    let gravity = terrain.map_or(9.81, |t| t.globe().gravity() as f32);
    burden.set_if_neq(Burden::of(pack.mass(), gravity));
}
