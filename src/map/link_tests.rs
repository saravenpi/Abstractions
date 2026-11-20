//! Tests of the map on a server: the active pins that come in and the one that goes out.

use std::sync::{Arc, Mutex};

use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{ClientMsg, ServerMsg, Waypoint as Wire, Welcome};
use bevy::math::DVec3;
use bevy::prelude::*;

use super::{receive, to_wire};
use crate::map::places::Places;
use crate::net::{FromServer, Link, Role, Script, Source};
use crate::saves::Pin;
use crate::universe::Universe;

/// Returns a pin on the home planet of seed 3 toward `dir`.
fn spot(dir: DVec3) -> Pin {
    Pin {
        name: String::new(),
        body: Universe::new(3).home(),
        dir: dir.to_array(),
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

/// Returns a headless app that takes in waypoints over a scripted link as player 3, with the
/// pins `places`, and the script.
fn joined(places: Places) -> (App, Script) {
    let script: Script = Arc::new(Mutex::new(Default::default()));
    let mut link = Link::over(Source::Script(script.clone()), Role::Host);
    link.you = Some(PlayerId(3));
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<FromServer>()
        .insert_resource(link)
        .insert_resource(places)
        .add_systems(Update, receive);
    (app, script)
}

/// Hands `msg` to the app as if the server sent it, and runs a frame.
fn hear(app: &mut App, msg: ServerMsg) {
    app.world_mut().write_message(FromServer(msg));
    app.update();
}

/// The active pin kept from before goes to the server with its welcome, which knows none.
#[test]
fn a_kept_active_pin_is_sent_again_on_welcome() {
    let mine = spot(DVec3::X);
    let (mut app, script) = joined(Places {
        pins: vec![mine.clone()],
        active: Some(0),
        ..Places::default()
    });
    hear(&mut app, welcome(Vec::new()));
    let sent = script.lock().unwrap().sent.clone();
    assert_eq!(sent, vec![ClientMsg::Waypoint(Some(to_wire(&mine)))]);
}

/// The welcome brings the others' waypoints and never this player's own; a cleared waypoint
/// goes, and a change about this player is not taken as a friend's.
#[test]
fn waypoints_come_in_without_your_own() {
    let (mut app, _) = joined(Places::default());
    let (lin, you) = (spot(DVec3::Y), spot(DVec3::Z));
    let given = vec![(PlayerId(1), to_wire(&lin)), (PlayerId(3), to_wire(&you))];
    hear(&mut app, welcome(given));
    let places = app.world().resource::<Places>();
    assert_eq!(places.me, 3);
    assert_eq!(places.friends.len(), 1);
    assert!(DVec3::from_array(places.friends[&1].dir).distance(DVec3::Y) < 1e-6);
    let about_you = ServerMsg::Waypoint {
        player: PlayerId(3),
        waypoint: Some(to_wire(&you)),
    };
    hear(&mut app, about_you);
    assert!(!app.world().resource::<Places>().friends.contains_key(&3));
    let cleared = ServerMsg::Waypoint {
        player: PlayerId(1),
        waypoint: None,
    };
    hear(&mut app, cleared);
    assert!(app.world().resource::<Places>().friends.is_empty());
}
