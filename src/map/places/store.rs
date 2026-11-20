//! Keeping the pins: loaded from the active world and written back into it, drawn as markers
//! for the compass, the minimap and the map, and forgotten when their world or session ends.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Places;
use crate::computer::Computer;
use crate::map::palette::player_color;
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::net::Link;
use crate::saves::{ActiveWorld, Pin};
use crate::screens::Screen as Shown;
use crate::ui::theme;
use crate::universe::{BodyId, Universe};

/// Reads the pins of the world that was just made active. Old worlds kept them as favourites,
/// in the same record, so they come in as pins.
pub(in crate::map) fn load(active: Option<Res<ActiveWorld>>, mut places: ResMut<Places>) {
    let Some(active) = active.filter(|active| active.is_added()) else {
        return;
    };
    places.pins.clone_from(&active.0.save.marks);
    places.active = None;
    places.selected = None;
}

/// Writes the pins into the active world whenever they change, so the next save keeps them.
pub(in crate::map) fn store(places: Res<Places>, active: Option<ResMut<ActiveWorld>>) {
    let Some(mut active) = active.filter(|_| places.is_changed()) else {
        return;
    };
    if active.0.save.marks != places.pins {
        active.0.save.marks.clone_from(&places.pins);
    }
}

/// Returns the marker of `pin` of the player `owner`, drawn as `kind` in `color`, on a body
/// `radius` metres wide.
fn marker(pin: &Pin, kind: MarkerKind, (owner, color): (u32, Color), radius: f64) -> Marker {
    Marker {
        kind,
        place: MarkerPlace::Body(pin.body, DVec3::from_array(pin.dir) * radius),
        color,
        label: pin.name.clone(),
        owner,
    }
}

/// Writes every pin as a marker when one changed, or when the universe they lie in appears: a
/// welcome sets them before it exists. The active pin is a waypoint marker and the others
/// favourite markers, which the compass shows only near; the active pins of the others are
/// waypoint markers of their player.
pub(in crate::map) fn fill(
    places: Res<Places>,
    universe: Option<Res<Universe>>,
    mut markers: ResMut<Markers>,
) {
    let due = |universe: &Res<Universe>| places.is_changed() || universe.is_changed();
    let Some(universe) = universe.filter(due) else {
        return;
    };
    markers.clear_kind(MarkerKind::Waypoint);
    markers.clear(MarkerKind::Favourite, 0);
    let radius = |body: BodyId| universe.planet(body).radius;
    for (index, pin) in places.pins.iter().enumerate() {
        let kind = match places.active == Some(index) {
            true => MarkerKind::Waypoint,
            false => MarkerKind::Favourite,
        };
        markers
            .0
            .push(marker(pin, kind, (0, theme::PIN), radius(pin.body)));
    }
    for (owner, pin) in &places.friends {
        let theirs = (*owner, player_color(*owner));
        let mark = marker(pin, MarkerKind::Waypoint, theirs, radius(pin.body));
        markers.0.push(mark);
    }
}

/// Forgets every pin and its marker: the world they were kept in ended.
fn reset(mut places: ResMut<Places>, mut markers: ResMut<Markers>) {
    *places = Places::default();
    markers.clear_kind(MarkerKind::Waypoint);
    markers.clear_kind(MarkerKind::Favourite);
}

/// Forgets the pins of the other players and their markers once the session ended. The pins of
/// this player stay with the world, and the next server gets the active one with its welcome.
fn part(mut places: ResMut<Places>, mut markers: ResMut<Markers>) {
    places.me = 0;
    places.friends.clear();
    markers
        .0
        .retain(|marker| marker.kind != MarkerKind::Waypoint || marker.owner == 0);
}

/// Lets go of the selected pin: its card shows only while the map is open.
fn unselect(mut places: ResMut<Places>) {
    if places.selected.is_some() {
        places.selected = None;
    }
}

/// Registers the end of pins: all of them with the world, the others' with the session, and the
/// selection with the map.
pub(in crate::map) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Shown::Title), reset)
        .add_systems(OnEnter(Computer::Open), unselect)
        .add_systems(OnExit(Computer::Open), unselect)
        .add_systems(Update, part.run_if(resource_removed::<Link>));
}
