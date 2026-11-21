//! The height of the pins: a pin lies on the ground or the sea of its body, read from the ground
//! of the planet the world streams. A pin dropped on the map of another planet, a pin of an older
//! file and the active pin of another player get theirs once the world comes to their planet.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Places;
use super::keys::drop_key;
use crate::menus::MarkerSet;
use crate::saves::Pin;
use crate::universe::BodyId;
use crate::world::{Globe, Planet};

/// Returns the height in metres above sea level of what a pin in unit direction `dir` lies on:
/// the ground, or the sea over it.
pub fn floor(globe: &Globe, dir: DVec3) -> f64 {
    f64::from(globe.height(dir).max(globe.sea_level()))
}

/// Returns true when `pin` lies on `body` and its height is not read yet.
fn unsettled(pin: &Pin, body: BodyId) -> bool {
    pin.body == body && pin.height.is_none()
}

/// Gives every pin of `places` on `body`, whose ground is `globe`, the height of the ground
/// under it, and returns true when one had none.
pub fn ground(places: &mut Places, body: BodyId, globe: &Globe) -> bool {
    let mut any = false;
    let Places { pins, friends, .. } = places;
    for pin in pins.iter_mut().chain(friends.values_mut()) {
        let dir = DVec3::from_array(pin.dir).try_normalize();
        if let Some(dir) = dir.filter(|_| pin.height.is_none() && pin.body == body) {
            pin.height = Some(floor(globe, dir));
            any = true;
        }
    }
    any
}

/// Gives the pins on the planet the world streams the height of the ground under them, once
/// each. It touches the pins only when one has no height, so their change still means a change.
pub(in crate::map) fn settle(planet: Option<Res<Planet>>, mut places: ResMut<Places>) {
    let Some(planet) = planet else {
        return;
    };
    let due = |pin: &Pin| unsettled(pin, planet.body);
    if places.pins.iter().chain(places.friends.values()).any(due) {
        ground(&mut places, planet.body, &planet.globe);
    }
}

/// Returns [`settle`] as it runs: on the planet the world streams, after the pin key and before
/// the markers are written, so a pin dropped this frame is drawn on the ground this frame.
pub(in crate::map) fn settled() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    settle
        .after(drop_key)
        .before(MarkerSet::Fill)
        .run_if(resource_exists::<Planet>)
}

#[cfg(test)]
#[path = "../places_ground_tests.rs"]
mod tests;
