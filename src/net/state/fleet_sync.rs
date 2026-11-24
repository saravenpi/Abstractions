//! Helper conversions and server event application for shared ship ownership sync.

use abstractions_net::wire::{self, ShipBody, ShipEvent};
use bevy::prelude::*;

use super::bodies::{Bodies, star_of};
use super::draw::drop_puppet;
use super::frame::{Place, Pose};
use super::store::Tracked;
use super::track::{Remotes, Track};
use crate::ship::{BoardGrant, Fleet, Ship, SwapShip};
use crate::sim::PlayerId;
use crate::space::Frame;
use crate::universe::Universe;

/// Ambient scene data passed into fleet event handlers.
pub(super) struct Context<'a> {
    /// Player id assigned by the server.
    pub(super) you: Option<PlayerId>,
    /// Local ship state, when spawned.
    pub(super) local_ship: Option<&'a Ship>,
    /// Render frame of the player anchor.
    pub(super) anchor_frame: Option<Frame>,
    /// Universe for celestial conversions.
    pub(super) universe: Option<&'a Universe>,
    /// Current universe clock in seconds.
    pub(super) clock: f64,
    /// Local network clock in seconds.
    pub(super) local: f64,
}

/// Seeds `remotes.ships[ship_id]` from local `ship` so swapping away or yielding never blinks.
pub(super) fn seed_track(
    ship_id: PlayerId,
    ship: &Ship,
    (commands, remotes, bodies): (&mut Commands, &mut Remotes, &mut Bodies),
    ctx: &Context<'_>,
) {
    if let Some(universe) = ctx.universe {
        bodies.sync(universe);
    }
    let from = Place::from(ship.frame);
    let Some(wire_frame) = bodies.choose((from, ship.pos), ship.landed, ctx.clock, star_of(from))
    else {
        return;
    };
    let pose = Pose {
        pos: ship.pos,
        vel: ship.vel,
        rot: ship.rot,
    };
    let Some(converted) = bodies.convert(pose, from, Place::from(wire_frame), ctx.clock) else {
        return;
    };
    let body = ShipBody {
        position: converted.pos,
        velocity: converted.vel.as_vec3(),
        rotation: converted.rot.as_quat(),
        throttle: ship.throttle as f32,
        gear: ship.gear as f32,
        damage: ship.damage as f32,
        landed: ship.landed,
        headlight: ship.headlight,
        ..ShipBody::default()
    };
    let mut track = Track::new(wire_frame);
    track.buffer.push(body.snap(ctx.clock), ctx.clock);
    track.latest = body;
    track.heard = ctx.local;
    if let Some(mut old) = remotes.ships.insert(ship_id, track) {
        drop_puppet(commands, &mut old);
    }
}

/// Builds `SwapShip` in a surface or target game frame from a remote `Track<ShipBody>`.
fn extract_swap(
    ship_id: PlayerId,
    track: &mut Track<ShipBody>,
    bodies: &mut Bodies,
    ctx: &Context<'_>,
) -> SwapShip {
    if let Some(universe) = ctx.universe {
        bodies.sync(universe);
    }
    let (pos, vel, rot, state) = match track.buffer.sample(ctx.clock) {
        Some(v) => (v.position, v.newer.velocity, v.rotation, v.newer),
        None => {
            let s = track.latest;
            (s.position, s.velocity, s.rotation, s)
        }
    };
    let dest = match track.frame {
        wire::Frame::Surface(body) => Frame::Surface(body),
        wire::Frame::Around(body) => Frame::Surface(body),
        wire::Frame::System(star) => ctx.anchor_frame.unwrap_or(Frame::System(star)),
    };
    let raw = Pose {
        pos,
        vel: vel.as_dvec3(),
        rot: rot.as_dquat(),
    };
    let pose = bodies
        .convert(raw, Place::from(track.frame), Place::from(dest), ctx.clock)
        .unwrap_or(raw);
    SwapShip {
        ship: ship_id,
        frame: dest,
        pos: pose.pos,
        vel: pose.vel,
        rot: pose.rot,
        gear: f64::from(state.gear),
        damage: f64::from(state.damage),
        landed: state.landed,
        headlight: state.headlight,
    }
}

/// Handles a server `Boarded` notification for local boarding, ship swapping or yielding.
fn handle_boarded(
    (ship, pilot): (PlayerId, PlayerId),
    (commands, remotes, bodies): (&mut Commands, &mut Remotes, &mut Bodies),
    fleet: &mut Fleet,
    ctx: &Context<'_>,
) {
    let Some(you) = ctx.you else {
        return;
    };
    if pilot == you {
        fleet.pending_board = None;
        if fleet.active == Some(ship) {
            fleet.granted = Some(BoardGrant::Current(ship));
            return;
        }
        if let (Some(old), Some(local_ship)) = (fleet.active, ctx.local_ship) {
            seed_track(old, local_ship, (commands, remotes, bodies), ctx);
        }
        fleet.active = Some(ship);
        fleet.yield_local = false;
        let grant = remotes
            .ships
            .remove(&ship)
            .map_or(BoardGrant::Current(ship), |mut track| {
                let swap = extract_swap(ship, &mut track, bodies, ctx);
                drop_puppet(commands, &mut track);
                BoardGrant::Remote(Box::new(swap))
            });
        fleet.granted = Some(grant);
        return;
    }
    if fleet.active == Some(ship) {
        if let Some(local_ship) = ctx.local_ship {
            seed_track(ship, local_ship, (commands, remotes, bodies), ctx);
        }
        fleet.active = None;
        fleet.yield_local = true;
    }
}

/// Applies one server `ShipEvent` to `remotes` and optional `fleet`.
pub(super) fn apply_event(
    event: &ShipEvent,
    (commands, remotes, bodies): (&mut Commands, &mut Remotes, &mut Bodies),
    mut fleet: Option<&mut Fleet>,
    ctx: &Context<'_>,
) {
    match event {
        ShipEvent::World(list) => {
            remotes.pilots.clear();
            for item in list {
                remotes.pilots.insert(item.ship, item.pilot);
            }
            if let (Some(you), Some(fleet)) = (ctx.you, fleet.as_deref_mut())
                && fleet.active == Some(you)
                && remotes.pilot_of(you).is_some_and(|p| p != you)
            {
                if let Some(local_ship) = ctx.local_ship {
                    seed_track(you, local_ship, (commands, remotes, bodies), ctx);
                }
                fleet.active = None;
                fleet.yield_local = true;
            }
        }
        ShipEvent::Boarded { ship, pilot } => {
            remotes.pilots.retain(|_, p| *p != *pilot);
            remotes.pilots.insert(*ship, *pilot);
            if let Some(fleet) = fleet {
                handle_boarded((*ship, *pilot), (commands, remotes, bodies), fleet, ctx);
            }
        }
        ShipEvent::Left { ship, pilot } => {
            if remotes.pilots.get(ship) == Some(pilot) {
                remotes.pilots.remove(ship);
            }
        }
        ShipEvent::Denied(ship) => {
            if let Some(fleet) = fleet.filter(|f| f.pending_board == Some(*ship)) {
                fleet.pending_board = None;
            }
        }
        ShipEvent::Gone(ship) => {
            remotes.pilots.remove(ship);
            if let Some(mut track) = remotes.ships.remove(ship) {
                drop_puppet(commands, &mut track);
            }
        }
    }
}
