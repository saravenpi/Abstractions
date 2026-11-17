//! The ship in the save file: its record is written into the active world and read back from it.

use bevy::prelude::*;

use super::body::{Ship, ShipState};
use crate::saves::ActiveWorld;

/// Seconds between two writes of the record of a moving ship.
const PERIOD: f32 = 0.25;

/// Returns the ship record stored in a world, when it has one that still reads.
pub fn saved(active: Option<&ActiveWorld>) -> Option<ShipState> {
    active?.0.save.ship.clone()?.try_into().ok()
}

/// Keeps the record of the ship in the active world current, so that every save holds it.
pub fn record(
    time: Res<Time>,
    ship: Single<Ref<Ship>>,
    active: Option<ResMut<ActiveWorld>>,
    mut last: Local<Option<(ShipState, f32)>>,
) {
    let (state, now) = (ship.state(), time.elapsed_secs());
    let fresh = last.is_none_or(|(old, at)| old.landed != state.landed || now - at >= PERIOD);
    let moved = last.is_none_or(|(old, _)| old != state);
    let Some(mut active) = active.filter(|_| ship.is_added() || (fresh && moved)) else {
        return;
    };
    active.0.save.ship = toml::Value::try_from(state).ok();
    *last = Some((state, now));
}
