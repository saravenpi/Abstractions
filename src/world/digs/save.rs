//! Persisting dug terrain craters in the active world save file.

use bevy::prelude::*;

use super::Digs;
use crate::saves::ActiveWorld;

/// Loads saved terrain dig craters when a world starts.
pub fn load(active: Option<Res<ActiveWorld>>, mut digs: ResMut<Digs>) {
    let mut loaded = Digs::default();
    if let Some(active) = active {
        loaded.set(active.0.save.digs.clone());
    }
    digs.set_if_neq(loaded);
}

/// Syncs modified terrain digs into the active save snapshot.
pub fn store(digs: Res<Digs>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| digs.is_changed()) else {
        return;
    };
    if active.0.save.digs.as_slice() != digs.all() {
        active.0.save.digs = digs.all().to_vec();
    }
}
