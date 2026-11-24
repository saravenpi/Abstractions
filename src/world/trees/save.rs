//! Persisting felled trees in the active save file.

use bevy::prelude::*;

use super::Felled;
use crate::saves::ActiveWorld;

/// Loads felled trees when starting a saved world.
pub fn load(active: Option<Res<ActiveWorld>>, mut felled: ResMut<Felled>) {
    let loaded = active.map_or_else(Felled::default, |a| Felled::from_text(&a.0.save.felled));
    felled.set_if_neq(loaded);
}

/// Syncs felled trees into the active save whenever [`Felled`] changes.
pub fn store(felled: Res<Felled>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| felled.is_changed()) else {
        return;
    };
    let text = felled.to_text();
    if active.0.save.felled != text {
        active.0.save.felled = text;
    }
}
