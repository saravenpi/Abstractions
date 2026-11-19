//! Waypoints: one point a player marks on a planet for everyone to see. The waypoint key, or a
//! right click on the flat map, sets the player's own; the same again clears it.
//!
//! The key marks the place under the pointer on the flat map, and where the player is
//! anywhere else. On a server each change goes out as a [`WaypointChanged`], and the waypoints
//! of the other players come back into [`Waypoints::friends`]. Every waypoint is a marker in
//! the colour of its player.

use std::collections::HashMap;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::level::Level;
use super::me::Me;
use super::palette::player_color;
use super::surface::Surface;
use crate::computer::{Computer, Typing};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::net::Link;
use crate::screens::Screen as Shown;
use crate::settings::{Persistent, Settings};
use crate::universe::{BodyId, Universe};

/// One waypoint: a direction from the centre of a body, in the frame that turns with it.
#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    /// The planet or the moon it is on.
    pub body: BodyId,
    /// The unit direction from the centre of the body.
    pub dir: DVec3,
    /// Its name. Empty for none.
    pub name: String,
}

/// The waypoint of this player and those of the others.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Waypoints {
    /// The id of this player on the server, which picks its colour; 0 alone.
    pub me: u32,
    /// The waypoint of this player.
    pub mine: Option<Waypoint>,
    /// The waypoints of the other players, by their id.
    pub friends: HashMap<u32, Waypoint>,
}

/// Sent whenever this player sets or clears their waypoint, for the server.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct WaypointChanged(pub Option<Waypoint>);

/// Sets the waypoint of this player at `at`, or clears it when one is set, and returns what
/// changed for the server. Nothing changes without a place.
pub fn toggle(waypoints: &mut Waypoints, at: Option<(BodyId, DVec3)>) -> Option<WaypointChanged> {
    if waypoints.mine.take().is_some() {
        return Some(WaypointChanged(None));
    }
    let (body, at) = at?;
    let dir = at.try_normalize()?;
    let set = Waypoint {
        body,
        dir,
        name: String::new(),
    };
    waypoints.mine = Some(set.clone());
    Some(WaypointChanged(Some(set)))
}

/// What the waypoint key reads.
type Asked<'w> = (
    Res<'w, ButtonInput<KeyCode>>,
    Res<'w, ButtonInput<MouseButton>>,
    Res<'w, Persistent<Settings>>,
);

/// Where the map is and what is typed.
type Screen<'w> = (
    Res<'w, State<Computer>>,
    Res<'w, Typing>,
    Res<'w, Level>,
    Res<'w, Surface>,
);

/// Sets or clears the waypoint on the waypoint key, or on a right click on the flat map: at the
/// place under the pointer on the flat map, else where the player is. While a search or a name
/// is typed the key types instead.
pub(super) fn drop_key(
    (keys, mouse, settings): Asked,
    (state, typing, level, surface): Screen,
    me: Me,
    mut waypoints: ResMut<Waypoints>,
    mut changes: MessageWriter<WaypointChanged>,
) {
    let open = *state.get() == Computer::Open;
    let key = keys.just_pressed(settings.controls.keys.waypoint) && typing.is_off();
    let on_map = match *level {
        Level::Planet(body) if open && surface.over => surface.point.map(|dir| (body, dir)),
        _ => None,
    };
    let right = mouse.just_pressed(MouseButton::Right) && on_map.is_some();
    if !(key || right) {
        return;
    }
    if let Some(change) = toggle(&mut waypoints, on_map.or_else(|| me.spot())) {
        changes.write(change);
    }
}

/// Returns the marker of a waypoint of the player `owner` on a body `radius` metres wide.
fn marker(waypoint: &Waypoint, owner: u32, radius: f64) -> Marker {
    Marker {
        kind: MarkerKind::Waypoint,
        place: MarkerPlace::Body(waypoint.body, waypoint.dir * radius),
        color: player_color(owner),
        label: waypoint.name.clone(),
        owner,
    }
}

/// Writes every waypoint as a marker when one changed, or when the universe they lie in
/// appears: a welcome sets them before it exists. The waypoints of the others keep the
/// owner of their player; this player's own the owner 0 of the game.
pub(super) fn fill(
    waypoints: Res<Waypoints>,
    universe: Option<Res<Universe>>,
    mut markers: ResMut<Markers>,
) {
    let due = |universe: &Res<Universe>| waypoints.is_changed() || universe.is_changed();
    let Some(universe) = universe.filter(due) else {
        return;
    };
    markers.clear_kind(MarkerKind::Waypoint);
    let radius = |body: BodyId| universe.planet(body).radius;
    if let Some(mine) = &waypoints.mine {
        let mut own = marker(mine, waypoints.me, radius(mine.body));
        own.owner = 0;
        markers.0.push(own);
    }
    for (owner, waypoint) in &waypoints.friends {
        markers
            .0
            .push(marker(waypoint, *owner, radius(waypoint.body)));
    }
}

/// Forgets every waypoint and its marker: the world they were set in ended.
fn reset(mut waypoints: ResMut<Waypoints>, mut markers: ResMut<Markers>) {
    *waypoints = Waypoints::default();
    markers.clear_kind(MarkerKind::Waypoint);
}

/// Forgets the waypoints of the other players and their markers once the session ended. This
/// player's own stays with the world, and the next server gets it with its welcome.
fn part(mut waypoints: ResMut<Waypoints>, mut markers: ResMut<Markers>) {
    waypoints.me = 0;
    waypoints.friends.clear();
    markers
        .0
        .retain(|marker| marker.kind != MarkerKind::Waypoint || marker.owner == 0);
}

/// Registers the end of waypoints: all of them with the world, the others' with the session.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Shown::Title), reset)
        .add_systems(Update, part.run_if(resource_removed::<Link>));
}

#[cfg(test)]
#[path = "waypoint_tests.rs"]
mod tests;
