//! Keeping the condition of the robot in the world file, so a hurt robot stays hurt.

use bevy::prelude::*;

use crate::condition::Condition;
use crate::saves::ActiveWorld;

/// Reads the condition of the world that was just made active: whole when its file has none.
pub fn load(active: Option<Res<ActiveWorld>>, mut condition: ResMut<Condition>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    *condition = active
        .0
        .save
        .health
        .map_or_else(Condition::default, Condition::restored);
}

/// Writes the condition into the active world whenever it changes, so the next save keeps it.
pub fn store(condition: Res<Condition>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active else { return };
    let saved = Some(condition.saved());
    if active.0.save.health != saved {
        active.0.save.health = saved;
    }
}
