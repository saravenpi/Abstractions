//! Client-side tests of shared ships: boarding a friend's ship, yielding ours when boarded and
//! keeping piloted ships alive after an owner leaves.

use abstractions_net::io::ClientEvent;
use abstractions_net::wire::{Body, Head, Kind, Record, ServerMsg, ShipBody, ShipEvent, push};
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::stage::{YOU, app, frame_at, ground, linked};
use crate::net::link::Script;
use crate::net::state::track::Remotes;
use crate::ship::{BoardGrant, Fleet, Ship};
use crate::sim::PlayerId;
use crate::space::{Frame, Placed};

/// Builds a `Kind::Ship` datagram packet for `ship` on `body` at `pos` sampled at `time`.
fn ship_batch(
    ship: PlayerId,
    frame: abstractions_net::wire::Frame,
    pos: DVec3,
    time: f64,
) -> Vec<u8> {
    let body = ShipBody {
        position: pos,
        landed: true,
        gear: 1.0,
        ..ShipBody::default()
    };
    let mut bytes = Vec::new();
    body.encode(&frame, &mut bytes);
    let head = Head {
        player: ship,
        kind: Kind::Ship,
        frame,
        time,
        flags: 0,
    };
    let mut out = Vec::new();
    push(&mut out, &Record { head, body: &bytes });
    out
}

/// Initializes a test `App` with `Fleet`, a local `Ship` on the home ground and an active `Link`.
fn ship_app(tag: &str) -> (App, Script, abstractions_net::ids::BodyId, DVec3) {
    let mut app = app(tag);
    let (home, spot) = ground();
    let ship_pos = spot + DVec3::Y * 2.0;
    app.init_resource::<Fleet>().world_mut().spawn((
        Ship::at_rest(Frame::Surface(home), ship_pos, DQuat::IDENTITY),
        Placed {
            pos: ship_pos,
            rot: DQuat::IDENTITY,
        },
        Visibility::default(),
    ));
    let script = linked(&mut app);
    (app, script, home, spot)
}

/// Boarding a remote ship grants `BoardGrant::Remote`, seeds our old ship track and survives
/// owner disconnect, while an observed friend flying another leaving player's ship stays tracked.
#[test]
fn boarding_remote_ship_seeds_local_and_survives_owner_leave() {
    let (mut app, script, home, spot) = ship_app("fleet-board");
    let (friend, third) = (PlayerId(7), PlayerId(8));
    let wire_frame = abstractions_net::wire::Frame::Surface(home);
    let pos = (spot + DVec3::X * 25.0).normalize() * (spot.length() + 2.0);
    for step in 0..5 {
        let t = 500.0 + f64::from(step) / 30.0;
        let mut wire = script.lock().unwrap();
        wire.events.push_back(ClientEvent::Datagram(ship_batch(
            friend, wire_frame, pos, t,
        )));
        wire.events
            .push_back(ClientEvent::Datagram(ship_batch(third, wire_frame, pos, t)));
        drop(wire);
        frame_at(&mut app, t);
    }
    let mut wire = script.lock().unwrap();
    wire.events
        .push_back(ClientEvent::Message(ServerMsg::Ship(ShipEvent::Boarded {
            ship: friend,
            pilot: YOU,
        })));
    wire.events
        .push_back(ClientEvent::Message(ServerMsg::Ship(ShipEvent::Boarded {
            ship: third,
            pilot: friend,
        })));
    drop(wire);
    frame_at(&mut app, 500.2);
    let fleet = app.world().resource::<Fleet>();
    assert_eq!(fleet.active, Some(friend));
    assert!(matches!(fleet.granted, Some(BoardGrant::Remote(_))));
    assert!(app.world().resource::<Remotes>().ships.contains_key(&YOU));
    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Message(ServerMsg::Left(third)));
    frame_at(&mut app, 500.3);
    let remotes = app.world().resource::<Remotes>();
    assert!(remotes.ships.contains_key(&third));
    assert_eq!(remotes.pilot_of(third), Some(friend));
    assert!(app.world().resource::<Fleet>().active == Some(friend));
}

/// When another player boards our active ship, `Fleet` clears `active`, flags `yield_local` and
/// seeds a remote track at the local ship's pose.
#[test]
fn remote_pilot_boarding_local_ship_sets_yield_local() {
    let (mut app, script, _, _) = ship_app("fleet-yield");
    frame_at(&mut app, 500.0);
    let friend = PlayerId(7);
    let stolen = ServerMsg::Ship(ShipEvent::Boarded {
        ship: YOU,
        pilot: friend,
    });
    script
        .lock()
        .unwrap()
        .events
        .push_back(ClientEvent::Message(stolen));
    frame_at(&mut app, 500.1);
    let fleet = app.world().resource::<Fleet>();
    assert_eq!(fleet.active, None);
    assert!(fleet.yield_local);
    assert!(app.world().resource::<Remotes>().ships.contains_key(&YOU));
}
