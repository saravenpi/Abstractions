//! Client-side systems synchronizing shared ship seats, boarding/leaving requests and yield state.

use abstractions_net::wire::{ClientMsg, ServerMsg, ShipAct};
use avian3d::prelude::ColliderDisabled;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Remote;
use super::bodies::Bodies;
use super::fleet_sync::{Context, apply_event};
use super::track::Remotes;
use crate::net::link::{FromServer, Link};
use crate::player::Player;
use crate::ship::{Fleet, Ship};
use crate::sim::PlayerId;
use crate::space::{Anchor, Clock, Location, Placed, Travel};
use crate::universe::Universe;

/// Distance in metres within which a player on foot can board a parked remote ship.
const BOARD_RANGE: f32 = 7.0;

/// Ambient scene data needed by `hear` to convert remote ships and detect nearby hatches.
#[derive(SystemParam)]
pub struct Scene<'w, 's> {
    /// Active universe specification.
    universe: Option<Res<'w, Universe>>,
    /// Current render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Player planet and travel state.
    location: Option<Res<'w, Location>>,
    /// Universe clock in seconds.
    clock: Res<'w, Clock>,
    /// Local ship state, when spawned.
    local_ship: Query<'w, 's, &'static Ship>,
    /// Local player body transform.
    player: Query<'w, 's, &'static Transform, With<Player>>,
    /// Remote entity transforms and planet-frame poses.
    remote_placed: Query<'w, 's, &'static Placed, With<Remote>>,
}

/// Returns the owner id of the nearest landed unpiloted remote ship within [`BOARD_RANGE`] when on foot.
fn nearest_remote(remotes: &Remotes, scene: &Scene) -> Option<PlayerId> {
    let anchor = scene.anchor.as_deref()?;
    let travel = scene.location.as_deref().map(|l| l.travel)?;
    if travel != Travel::OnFoot {
        return None;
    }
    let body_pos = scene.player.single().ok()?.translation;
    let mut best = (BOARD_RANGE, None);
    for (&ship_id, track) in &remotes.ships {
        if !track.latest.landed || remotes.pilots.contains_key(&ship_id) {
            continue;
        }
        let Some(placed) = track.puppet.and_then(|e| scene.remote_placed.get(e).ok()) else {
            continue;
        };
        let dist = body_pos.distance(anchor.to_render(placed.pos));
        if dist < best.0 {
            best = (dist, Some(ship_id));
        }
    }
    best.1
}

/// Applies incoming server ship pilot events, syncs session start and finds nearby remote ships.
pub fn hear(
    mut commands: Commands,
    (mut link, mut remotes, mut bodies): (ResMut<Link>, ResMut<Remotes>, ResMut<Bodies>),
    mut fleet: Option<ResMut<Fleet>>,
    mut from: MessageReader<FromServer>,
    scene: Scene,
) {
    let ctx = Context {
        you: link.you,
        local_ship: scene.local_ship.single().ok(),
        anchor_frame: scene.anchor.as_deref().map(|a| a.frame),
        universe: scene.universe.as_deref(),
        clock: scene.clock.seconds,
        local: link.now(),
    };
    if let Some(fleet) = fleet.as_deref_mut() {
        fleet.online = true;
        if let Some(you) = link.you
            && fleet.joined != Some(you)
        {
            fleet.joined = Some(you);
            fleet.active = Some(you);
            fleet.yield_local = false;
            if scene
                .location
                .as_deref()
                .is_some_and(|l| l.travel == Travel::InShip)
            {
                link.send(ClientMsg::Ship(ShipAct::Board(you)));
            }
        }
    }
    for FromServer(msg) in from.read() {
        if let ServerMsg::Ship(event) = msg {
            let parts = (&mut commands, remotes.as_mut(), bodies.as_mut());
            apply_event(event, parts, fleet.as_deref_mut(), &ctx);
        }
    }
    if let Some(fleet) = fleet.as_deref_mut() {
        fleet.near_remote = nearest_remote(&remotes, &scene);
    }
}

/// Sends queued `Board` and `Leave` acts placed by `ship::board::board` in the same frame.
pub fn act(mut link: ResMut<Link>, mut fleet: Option<ResMut<Fleet>>) {
    let Some(fleet) = fleet.as_deref_mut() else {
        return;
    };
    if let Some(ship) = fleet.pending_board.take() {
        link.send(ClientMsg::Ship(ShipAct::Board(ship)));
    }
    if let Some(ship) = fleet.pending_leave.take() {
        link.send(ClientMsg::Ship(ShipAct::Leave(ship)));
    }
}

/// Resets shared fleet state and restores local ship visibility/collider when leaving a session.
pub fn reset(
    mut commands: Commands,
    mut remotes: ResMut<Remotes>,
    mut fleet: Option<ResMut<Fleet>>,
    mut ships: Query<(Entity, &mut Visibility), With<Ship>>,
) {
    remotes.pilots.clear();
    if let Some(fleet) = fleet.as_deref_mut().filter(|f| f.online) {
        *fleet = Fleet::default();
        if let Ok((entity, mut vis)) = ships.single_mut() {
            *vis = Visibility::Inherited;
            commands.entity(entity).remove::<ColliderDisabled>();
        }
    }
}
