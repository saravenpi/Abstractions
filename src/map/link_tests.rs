//! Tests of the map on a server: the waypoints that come in and the one that goes out.

use std::sync::{Arc, Mutex};

use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{ClientMsg, ServerMsg, Waypoint as Wire, Welcome};
use bevy::math::DVec3;
use bevy::prelude::*;

use super::{receive, to_wire};
use crate::map::waypoint::{Waypoint, Waypoints};
use crate::net::{FromServer, Link, Role, Script, Source};
use crate::universe::Universe;

/// Returns a waypoint on the home planet of seed 3 toward `dir`.
fn spot(dir: DVec3) -> Waypoint {
    Waypoint {
        body: Universe::new(3).home(),
        dir,
        name: String::new(),
    }
}

/// Returns a welcome for player 3 with the waypoints `waypoints`.
fn welcome(waypoints: Vec<(PlayerId, Wire)>) -> ServerMsg {
    ServerMsg::Welcome(Welcome {
        you: PlayerId(3),
        seed: 3,
        clock: 0.0,
        changes: Changes::default(),
        atlas: String::new(),
        record: None,
        players: Vec::new(),
        explored: Vec::new(),
        waypoints,
    })
}

/// Returns a headless app that takes in waypoints over a scripted link as player 3, with
/// `waypoints`, and the script.
fn joined(waypoints: Waypoints) -> (App, Script) {
    let script: Script = Arc::new(Mutex::new(Default::default()));
    let mut link = Link::over(Source::Script(script.clone()), Role::Host);
    link.you = Some(PlayerId(3));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<FromServer>()
        .insert_resource(link)
        .insert_resource(waypoints)
        .add_systems(Update, receive);
    (app, script)
}

/// Hands `msg` to the app as if the server sent it, and runs a frame.
fn hear(app: &mut App, msg: ServerMsg) {
    app.world_mut().write_message(FromServer(msg));
    app.update();
}

/// A waypoint kept from before goes to the server with its welcome, which knows none.
#[test]
fn a_kept_waypoint_is_sent_again_on_welcome() {
    let mine = spot(DVec3::X);
    let (mut app, script) = joined(Waypoints {
        mine: Some(mine.clone()),
        ..Waypoints::default()
    });
    hear(&mut app, welcome(Vec::new()));
    let sent = script.lock().unwrap().sent.clone();
    assert_eq!(sent, vec![ClientMsg::Waypoint(Some(to_wire(&mine)))]);
}
