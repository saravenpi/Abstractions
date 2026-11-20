//! Keeping the taken rocks in the world file.

use bevy::prelude::*;

use super::Taken;
use crate::saves::ActiveWorld;

/// Reads the taken rocks of the world that is starting, or none when no world file is active.
pub fn load(active: Option<Res<ActiveWorld>>, mut taken: ResMut<Taken>) {
    let loaded = active.map_or_else(Taken::default, |a| Taken::from_text(&a.0.save.taken));
    taken.set_if_neq(loaded);
}

/// Writes the taken rocks into the active world whenever they change, so the next save keeps them.
pub fn store(taken: Res<Taken>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| taken.is_changed()) else {
        return;
    };
    let text = taken.to_text();
    if active.0.save.taken != text {
        active.0.save.taken = text;
    }
}
