//! The ships of the other players: a hull dressed from the shared ship look, with no flight
//! state, no physics and no helm. The legs follow the gear of the records, the stick and the
//! lever what the pilot holds, the gear lever and the warp switch its gear and its drive, the
//! instruments its speed, its drive and its gear. The engine
//! glow and the exhaust, of a material of its own, follow its throttle, boost and warp drive.

use abstractions_net::wire::{ShipBody, WARP_ACTIVE, WARP_SPOOLING};
use bevy::prelude::*;

use super::{Eye, Sight, drop_puppet, seen, shown, visible};
use crate::net::state::Remote;
use crate::net::state::track::{Remotes, Track};
use crate::screens::Screen;
use crate::ship::cockpit::{Cockpit, Levers, Readings, Seen};
use crate::ship::{self, Engine, Leg, SEAT, level_of, reach, top_speed};
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
        &'static mut Engine,
    ),
    With<Remote>,
>;

/// The seat of a ship puppet: the child of the hull at the eyes of its pilot, which a seated
/// robot hangs from.
#[derive(Component, Debug, Clone, Copy)]
pub struct Seat(pub Entity);

/// The legs of every ship, with the ship each hangs from.
type Legs<'w, 's> = Query<'w, 's, (&'static ChildOf, &'static mut Transform), With<Leg>>;

/// What spawning a ship puppet needs.
type Spawner<'w, 's> = (
    Commands<'w, 's>,
    Res<'w, ship::Looks>,
    ResMut<'w, Assets<StandardMaterial>>,
);

/// Spawns the puppet of a ship at `placed` and returns it. Its engine glow is a copy of the
/// shared one, so it lights with this hull alone, and it has a seat a robot can sit in.
fn spawn((commands, looks, materials): &mut Spawner, eye: &Eye, placed: Placed) -> Entity {
    let shared = materials.get(&looks.skins[GLOW]).cloned();
    let glow = shared.map_or_else(|| looks.skins[GLOW].clone(), |copy| materials.add(copy));
    let (at, _) = seen(eye, &placed);
    let marks = (
        Remote,
        placed,
        at,
        Visibility::default(),
        Engine::new(glow.clone()),
        DespawnOnEnter(Screen::Title),
    );
    let place = Transform::from_translation(SEAT.as_vec3());
    let seat = commands.spawn((Name::new("Remote seat"), place, Visibility::default()));
    let seat = seat.id();
    let mut hull = commands.spawn((Name::new("Remote ship"), marks, Seat(seat)));
    hull.add_child(seat);
    ship::dress(&mut hull, looks, glow);
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

/// Returns how far the warp drive of `state` is: none when off, the share of the spool while
/// it spools, all of it in warp.
fn drive(state: &ShipBody) -> f32 {
    match state.warp {
        WARP_ACTIVE => 1.0,
        WARP_SPOOLING => state.warp_level,
        _ => 0.0,
    }
}

/// Returns where the warp switch of a ship in `state` stands: off, spooling or engaged.
fn switch(state: &ShipBody) -> f32 {
    match state.warp {
        WARP_ACTIVE => 1.0,
        WARP_SPOOLING => 0.5,
        _ => 0.0,
    }
}

/// Returns what the instruments of a ship in `state` read, `clearance` metres above the
/// ground: its speed against the top speed there, or its warp speed in warp, the charge of
/// the drive, the gear and the throttle. It carries no attitude and no course, so those
/// instruments stay dark.
fn dials(state: &ShipBody, clearance: f64) -> Readings {
    let warping = state.warp == WARP_ACTIVE;
    let speed = f64::from(state.velocity.length());
    let load = match warping {
        true => state.warp_level,
        false => (speed / top_speed(clearance)) as f32,
    };
    Readings::seen(Seen {
        speed: (!warping).then_some(speed),
        load,
        throttle: state.throttle.max(0.0),
        clearance,
        drive: drive(state),
        gear: state.gear,
    })
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
    let Ok((mut at, mut cockpit, mut shown, mut engine)) = hulls.get_mut(entity) else {
        track.puppet = None;
        return;
    };
    *at = placed;
    let state = view.newer;
    let gear = view.older.gear + (state.gear - view.older.gear) * view.blend;
    fold(legs, entity, gear);
    let [pitch, roll] = state.stick;
    cockpit.asked = Levers {
        stick: Vec3::new(pitch, 0.0, roll),
        throttle: state.throttle.max(0.0),
        gear,
        warp: switch(&state),
    };
    cockpit.read = dials(&state, sight.clearance(track.frame, view.position));
    engine.level = level_of(state.throttle, state.boost, drive(&state));
    engine.warp = drive(&state);
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
