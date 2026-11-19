//! The ships of the other players: a hull dressed from the shared ship look, with no flight
//! state, no physics and no helm. The legs follow the gear of the records and the lever the
//! throttle; the engine glow and the exhaust stay as they are at idle.

use abstractions_net::wire::ShipBody;
use bevy::prelude::*;

use super::{Eye, Sight, drop_puppet, seen, shown, visible};
use crate::net::state::Remote;
use crate::net::state::track::{Remotes, Track};
use crate::screens::Screen;
use crate::ship::cockpit::Cockpit;
use crate::ship::{self, Leg, reach};
use crate::space::Placed;

/// Rank of the engine glow among the skins of the shared ship look.
const GLOW: usize = 4;

/// The ship puppets.
type Hulls<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Placed,
        &'static mut Cockpit,
        &'static mut Visibility,
    ),
    With<Remote>,
>;

/// The legs of every ship, with the ship each hangs from.
type Legs<'w, 's> = Query<'w, 's, (&'static ChildOf, &'static mut Transform), With<Leg>>;

/// What spawning a ship puppet needs.
type Spawner<'w, 's> = (Commands<'w, 's>, Res<'w, ship::Looks>);

/// Spawns the puppet of a ship at `placed` and returns it.
fn spawn((commands, looks): &mut Spawner, eye: &Eye, placed: Placed) -> Entity {
    let (at, _) = seen(eye, &placed);
    let marks = (
        Remote,
        placed,
        at,
        Visibility::default(),
        DespawnOnEnter(Screen::Title),
    );
    let mut hull = commands.spawn((Name::new("Remote ship"), marks));
    ship::dress(&mut hull, looks, looks.skins[GLOW].clone());
    hull.id()
}

/// Sets the length of the legs of `hull` for the gear extension `gear`.
fn fold(legs: &mut Legs, hull: Entity, gear: f32) {
    let length = reach(f64::from(gear)) as f32;
    for (parent, mut leg) in legs.iter_mut() {
        if parent.parent() == hull && leg.scale.y != length {
            leg.scale.y = length;
        }
    }
}

/// Draws one ship of `track`.
fn draw(
    (spawner, hulls, legs): (&mut Spawner, &mut Hulls, &mut Legs),
    sight: &mut Sight,
    eye: &Eye,
    track: &mut Track<ShipBody>,
) {
    let Some((view, placed)) = sight.locate(track, eye) else {
        return;
    };
    let Some(entity) = track.puppet else {
        track.puppet = Some(spawn(spawner, eye, placed));
        return;
    };
    let Ok((mut at, mut cockpit, mut shown)) = hulls.get_mut(entity) else {
        track.puppet = None;
        return;
    };
    *at = placed;
    let state = view.newer;
    let gear = view.older.gear + (state.gear - view.older.gear) * view.blend;
    fold(legs, entity, gear);
    cockpit.asked.throttle = state.throttle.max(0.0);
    let (_, far) = seen(eye, &placed);
    shown.set_if_neq(visible(far, false));
}

/// Draws the ship of every other player at the body this player is at, and takes away the
/// puppets of the others.
pub fn ships(
    mut spawner: Spawner,
    mut sight: Sight,
    mut remotes: ResMut<Remotes>,
    mut hulls: Hulls,
    mut legs: Legs,
) {
    let Some(eye) = sight.eye() else {
        return;
    };
    for track in remotes.ships.values_mut() {
        if !shown(track.frame, eye.body) {
            drop_puppet(&mut spawner.0, track);
            continue;
        }
        draw(
            (&mut spawner, &mut hulls, &mut legs),
            &mut sight,
            &eye,
            track,
        );
    }
}
