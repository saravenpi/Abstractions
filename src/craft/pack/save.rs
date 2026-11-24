//! Keeping the pack in the world file, and weighing it under the gravity underfoot.

use bevy::prelude::*;

use super::{Burden, Pack};
use crate::condition::Condition;
use crate::saves::ActiveWorld;
use crate::world::Terrain;

/// Reads the pack of the world that was just made active and migrates any legacy layout to the slot grid.
pub(crate) fn load(active: Option<Res<ActiveWorld>>, mut pack: ResMut<Pack>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    *pack = active.0.save.pack.clone();
    pack.ensure_grid();
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

/// Weighs the pack under the gravity of the planet underfoot when the pack, the planet or the
/// torso of the robot changes: a hurt torso carries less.
pub(crate) fn weigh(
    pack: Res<Pack>,
    terrain: Option<Res<Terrain>>,
    condition: Option<Res<Condition>>,
    mut burden: ResMut<Burden>,
) {
    let moved = terrain.as_ref().is_some_and(|t| t.is_changed())
        || condition.as_ref().is_some_and(|c| c.is_changed());
    if !pack.is_changed() && !moved {
        return;
    }
    let gravity = terrain.map_or(9.81, |t| t.globe().gravity() as f32);
    let carry = condition.map_or(1.0, |condition| condition.carry());
    burden.set_if_neq(Burden::of(pack.mass(), gravity).strained(carry));
}
