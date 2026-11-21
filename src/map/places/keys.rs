//! The pin key, the arrival at the active pin, and the news of the active pin for the server.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{HERE, Places};
use crate::computer::Typing;
use crate::map::link::WaypointChanged;
use crate::map::me::Me;
use crate::net::SessionNotice;
use crate::saves::Pin;
use crate::settings::{Persistent, Settings};
use crate::universe::{BodyId, Universe};

/// What the pin key reads.
type Asked<'w> = (
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, Persistent<Settings>>,
    Res<'w, Typing>,
);

/// Applies the pin key at the point `at` of `body`, a body `radius` metres wide: the pin within
/// [`super::NEAR`] metres of it is removed, else a pin is dropped there, and a far pin never is
/// removed. Returns the line for the banner.
pub fn press(places: &mut Places, (body, at): (BodyId, DVec3), radius: f64) -> String {
    if let Some(index) = places.near(body, at, radius) {
        let gone = places.remove(index).unwrap_or_default();
        return format!("{} removed", gone.name);
    }
    match places.drop(body, at) {
        Some(index) => format!("{} placed", places.pins[index].name),
        None => "Pins full: remove one".to_string(),
    }
}

/// Drops a pin at the feet of the player, or at the ship while seated, on the pin key, or
/// removes the pin there. A search or a name being typed takes the key. The banner says what
/// happened.
pub(in crate::map) fn drop_key(
    (keys, settings, typing): Asked,
    me: Me,
    universe: Option<Res<Universe>>,
    mut places: ResMut<Places>,
    mut notices: MessageWriter<SessionNotice>,
) {
    if !(keys.just_pressed(settings.controls.keys.waypoint) && typing.is_off()) {
        return;
    }
    let (Some((body, at)), Some(universe)) = (me.spot(), universe) else {
        return;
    };
    let line = press(&mut places, (body, at), universe.planet(body).radius);
    notices.write(SessionNotice { line });
}

/// Run condition: a pin is the active one.
pub(in crate::map) fn leading(places: Res<Places>) -> bool {
    places.active.is_some()
}

/// Returns true when the player at the point `at` of `body`, a body `radius` metres wide, is
/// within [`HERE`] metres of `pin`.
pub fn reached(pin: &Pin, (body, at): (BodyId, DVec3), radius: f64) -> bool {
    pin.body == body && (at - pin.point(radius)).length() <= HERE
}

/// Lets go of the active pin once the player is within [`HERE`] metres of it. The pin stays,
/// and the banner says the player is there.
pub(in crate::map) fn arrive(
    me: Me,
    universe: Option<Res<Universe>>,
    mut places: ResMut<Places>,
    mut notices: MessageWriter<SessionNotice>,
) {
    let (Some((body, at)), Some(universe), Some(pin)) = (me.spot(), universe, places.active())
    else {
        return;
    };
    if !reached(pin, (body, at), universe.planet(body).radius) {
        return;
    }
    let line = format!("At {}", pin.name);
    places.set_active(None);
    notices.write(SessionNotice { line });
}

/// Tells the server whenever the active pin of this player changes.
pub(in crate::map) fn announce(
    places: Res<Places>,
    mut sent: Local<Option<Pin>>,
    mut changes: MessageWriter<WaypointChanged>,
) {
    if !places.is_changed() || places.active().cloned() == *sent {
        return;
    }
    *sent = places.active().cloned();
    changes.write(WaypointChanged(sent.clone()));
}
