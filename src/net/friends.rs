//! The other players as markers: each one at the newest place its records gave, in its own
//! colour and under its name, on this body, on another or in another system. The compass, the
//! minimap and the map read them from [`Markers`].

use abstractions_net::wire;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::Players;
use super::link::{Link, Receive};
use super::state::{Bodies, Place, Pose, Remotes};
use crate::map::palette::player_color;
use crate::menus::{Marker, MarkerKind, MarkerPlace, MarkerSet, Markers};
use crate::sim::PlayerId;
use crate::space::{Anchor, Clock};
use crate::universe::Universe;

/// What the markers of the other players are placed with.
type World<'w> = (
    ResMut<'w, Bodies>,
    Res<'w, Clock>,
    Option<Res<'w, Anchor>>,
    Option<Res<'w, Universe>>,
);

/// Returns where a thing at `pos` in the wire `frame` is shown: on its body in the turning
/// frame of that body, or in the frame of the `anchor` at `time` for a thing in a star frame.
fn place(
    bodies: &mut Bodies,
    (frame, pos): (wire::Frame, DVec3),
    anchor: &Anchor,
    time: f64,
) -> Option<MarkerPlace> {
    let pose = Pose {
        pos,
        vel: DVec3::ZERO,
        rot: DQuat::IDENTITY,
    };
    let (from, to) = (Place::from(frame), Place::from(anchor.frame));
    match frame {
        wire::Frame::Surface(body) => Some(MarkerPlace::Body(body, pos)),
        wire::Frame::Around(body) => bodies
            .convert(pose, from, Place::Surface(body), time)
            .map(|pose| MarkerPlace::Body(body, pose.pos)),
        wire::Frame::System(star) => {
            let galaxy = Place::Galaxy(star.galaxy);
            let direct = bodies.convert(pose, from, to, time);
            let far = |bodies: &mut Bodies| {
                let mid = bodies.convert(pose, from, galaxy, time)?;
                bodies.convert(mid, galaxy, to, time)
            };
            direct
                .or_else(|| far(bodies))
                .map(|pose| MarkerPlace::Anchor(pose.pos))
        }
    }
}

/// Writes `place` into the marker of the player `owner` named `name`, adding it the first time.
/// A marker already there is rewritten in place, so nothing is allocated once it exists.
fn mark(markers: &mut Markers, owner: u32, name: &str, place: MarkerPlace) {
    let own = |marker: &&mut Marker| marker.kind == MarkerKind::Friend && marker.owner == owner;
    if let Some(marker) = markers.0.iter_mut().find(own) {
        marker.place = place;
        if marker.label != name {
            name.clone_into(&mut marker.label);
        }
        return;
    }
    markers.0.push(Marker {
        kind: MarkerKind::Friend,
        place,
        color: player_color(owner),
        label: name.to_owned(),
        owner,
    });
}

/// Rewrites the marker of every other player at the newest place its records gave, and takes
/// away the markers of the players who left.
pub fn fill(
    (remotes, players): (Res<Remotes>, Res<Players>),
    world: World,
    mut markers: ResMut<Markers>,
) {
    let (mut bodies, clock, anchor, universe) = world;
    let (Some(anchor), Some(universe)) = (anchor.as_deref(), universe.as_deref()) else {
        return;
    };
    bodies.sync(universe);
    for (player, at) in remotes.whereabouts() {
        let Some(place) = place(&mut bodies, at, anchor, clock.seconds) else {
            continue;
        };
        let named = players.0.iter().find(|(id, _)| *id == player.0);
        mark(
            &mut markers,
            player.0,
            named.map_or("", |(_, name)| name),
            place,
        );
    }
    let here = |marker: &Marker| remotes.knows(PlayerId(marker.owner));
    markers
        .0
        .retain(|marker| marker.kind != MarkerKind::Friend || here(marker));
}

/// Takes away the markers of the other players once the session is over.
pub fn clear(mut markers: ResMut<Markers>) {
    if markers
        .0
        .iter()
        .any(|marker| marker.kind == MarkerKind::Friend)
    {
        markers.clear_kind(MarkerKind::Friend);
    }
}

/// Registers the markers of the other players: filled while linked, cleared after.
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            fill.after(Receive).run_if(resource_exists::<Link>),
            clear.run_if(not(resource_exists::<Link>)),
        )
            .in_set(MarkerSet::Fill)
            .distributive_run_if(resource_exists::<Markers>),
    );
}
