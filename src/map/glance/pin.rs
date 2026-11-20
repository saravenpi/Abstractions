//! Pins on the flat map: a right click on bare ground drops one there, and the marker a click
//! selects is the pin whose card shows.

use bevy::prelude::*;

use crate::map::level::Level;
use crate::map::pins::Pins;
use crate::map::places::Places;
use crate::map::surface::Surface;
use crate::menus::{MarkerKind, MarkerPlace, Markers};
use crate::net::SessionNotice;

/// Drops a pin at the place under the pointer on a right click on bare ground of the flat map.
/// A right click on a marker, or off the map, does nothing. The banner says what happened.
pub(in crate::map) fn drop_click(
    buttons: Res<ButtonInput<MouseButton>>,
    (level, surface): (Res<Level>, Res<Surface>),
    mut places: ResMut<Places>,
    mut notices: MessageWriter<SessionNotice>,
) {
    let (Level::Planet(body), Some(dir)) = (*level, surface.point) else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Right) || !surface.over || surface.hover.is_some() {
        return;
    }
    let line = match places.drop(body, dir) {
        Some(index) => format!("{} placed", places.pins[index].name),
        None => "Pins full: remove one".to_string(),
    };
    notices.write(SessionNotice { line });
}

/// Makes the pin whose marker was selected by a click the selected pin, and no pin when the
/// click let go of the selection or selected another marker. It reads the selection only when
/// that changes, so a pin picked in the list stays picked.
pub(in crate::map) fn pick(
    surface: Res<Surface>,
    (pins, markers): (Res<Pins>, Res<Markers>),
    mut places: ResMut<Places>,
    mut seen: Local<Option<usize>>,
) {
    if surface.chosen == *seen {
        return;
    }
    *seen = surface.chosen;
    let marker = surface
        .chosen
        .and_then(|chosen| pins.0.get(chosen)?.marker)
        .and_then(|index| markers.0.get(index))
        .filter(|marker| marker.owner == 0)
        .filter(|marker| matches!(marker.kind, MarkerKind::Waypoint | MarkerKind::Favourite));
    let at = match marker.map(|marker| marker.place) {
        Some(MarkerPlace::Body(body, pos)) => places.at(body, pos),
        _ => None,
    };
    places.selected = at;
}

#[cfg(test)]
#[path = "../glance_pin_tests.rs"]
mod tests;
