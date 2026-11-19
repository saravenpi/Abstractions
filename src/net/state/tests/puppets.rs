//! Remote robots in a headless app: two seconds of records of a walking player spawn one
//! puppet that follows the walk and goes when the player leaves; the records of this player
//! and of a player on another body spawn nothing.

use abstractions_net::io::ClientEvent;
use abstractions_net::wire::{Frame, ServerMsg};
use bevy::math::DVec3;

use super::stage::{YOU, app, frame_at, ground, linked, puppets, robot};
use crate::avatar::{AvatarPose, Mode};
use crate::net::Remote;
use crate::net::state::track::Remotes;
use crate::sim::PlayerId;
use crate::universe::BodyId;
use bevy::prelude::*;

/// Universe time the walk starts at.
const START: f64 = 500.0;

/// Returns where player 7 is at `time` and how fast it goes: east from the ground spot at 2 m/s.
fn walk(time: f64) -> (DVec3, DVec3) {
    let (_, spot) = ground();
    let east = DVec3::Y.cross(spot).normalize() * 2.0;
    (spot + east * (time - START), east)
}

/// Returns universe time at frame `step` of 30 a second.
fn at(step: u32) -> f64 {
    START + f64::from(step) / 30.0
}

/// Two seconds of records of player 7 spawn exactly one robot puppet with a pose, which
/// follows the walk a buffer delay behind; the player leaving takes it away.
#[test]
fn a_remote_robot_follows_its_walk_and_leaves() {
    let mut app = app("walk");
    let script = linked(&mut app);
    let (home, _) = ground();
    let send = |event| script.lock().unwrap().events.push_back(event);
    let mut first = None;
    for step in 0..60 {
        let batch = robot(PlayerId(7), Frame::Surface(home), at(step), walk(at(step)));
        send(ClientEvent::Datagram(batch));
        frame_at(&mut app, at(step));
        first = first.or(puppets(&mut app).first().copied());
    }
    let seen = puppets(&mut app);
    assert_eq!(seen.len(), 1);
    let delay = app.world().resource::<Remotes>().robots[&PlayerId(7)]
        .buffer
        .delay();
    let off = seen[0].pos.distance(walk(at(59) - delay).0);
    assert!(off < 0.02, "{off} m off the walk");
    assert!(first.unwrap().pos.distance(seen[0].pos) > 3.0);
    let mut poses = app
        .world_mut()
        .query_filtered::<&AvatarPose, With<Remote>>();
    assert_eq!(poses.single(app.world()).unwrap().mode, Mode::Walk);
    send(ClientEvent::Message(ServerMsg::Left(PlayerId(7))));
    frame_at(&mut app, at(60));
    frame_at(&mut app, at(61));
    assert!(puppets(&mut app).is_empty());
    assert!(app.world().resource::<Remotes>().robots.is_empty());
}

/// The records of this player come back from the server and spawn nothing; a player on
/// another body is kept in a buffer, with no puppet.
#[test]
fn own_records_and_other_bodies_spawn_nothing() {
    let mut app = app("unseen");
    let script = linked(&mut app);
    let (home, _) = ground();
    let moon = BodyId { moon: 1, ..home };
    for step in 0..30 {
        let mut wire = script.lock().unwrap();
        wire.events.push_back(ClientEvent::Datagram(robot(
            YOU,
            Frame::Surface(home),
            at(step),
            walk(at(step)),
        )));
        wire.events.push_back(ClientEvent::Datagram(robot(
            PlayerId(8),
            Frame::Surface(moon),
            at(step),
            walk(at(step)),
        )));
        drop(wire);
        frame_at(&mut app, at(step));
    }
    assert!(puppets(&mut app).is_empty());
    let remotes = app.world().resource::<Remotes>();
    assert!(remotes.robots.contains_key(&PlayerId(8)));
    assert!(!remotes.robots.contains_key(&YOU));
    assert_eq!(remotes.robots[&PlayerId(8)].puppet, None);
}
