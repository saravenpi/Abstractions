//! Tests of the sharing of the explored ground: the size of what goes out, the leaves kept
//! with no server, and the ground of a welcome.

use std::sync::{Arc, Mutex};

use abstractions_net::leaves::{Leaves, Run, key};
use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{CLIENT_MESSAGE_MAX, ClientMsg, ServerMsg, Welcome, encode};
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::{RUNS_MAX, messages};
use crate::map::explored::Explored;
use crate::net::{FromServer, Link, Role, Script, Source, Wire};
use crate::screens::Screen;
use crate::universe::{BodyId, Universe};

/// A long walk goes out as many messages, each one small enough for the server to read, and
/// every leaf once.
#[test]
fn a_big_send_is_cut_under_the_cap() {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    let ground = explored.ground(body).expect("room for a body");
    for step in 0..30_000u64 {
        ground.set(key(2, 15, step * 5));
    }
    let sent = messages(&mut explored);
    let mut leaves = 0;
    for msg in &sent {
        let size = encode(msg).expect("encodes").len();
        assert!(size < CLIENT_MESSAGE_MAX, "{size} bytes");
        let ClientMsg::Explored { runs, .. } = msg else {
            panic!("not explored leaves: {msg:?}");
        };
        assert!(runs.len() <= RUNS_MAX);
        leaves += Leaves::from_runs(runs).len();
    }
    assert_eq!(leaves, 30_000);
    assert!(sent.len() > 1);
    assert!(messages(&mut explored).is_empty());
}

/// Returns a headless app with the sharing of the explored ground and no server.
fn alone() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Screen>()
        .init_resource::<Explored>()
        .add_message::<FromServer>()
        .add_plugins(super::plugin);
    app.update();
    app
}

/// With no server the leaves uncovered here are not kept for one.
#[test]
fn nothing_piles_up_offline() {
    let mut app = alone();
    let body = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Explored>()
        .reveal(body, DVec3::X, 0.0001, 6.371e6);
    app.update();
    let mut explored = app.world_mut().resource_mut::<Explored>();
    assert!(
        explored
            .leaves(body)
            .is_some_and(|leaves| !leaves.is_empty())
    );
    assert!(explored.take_new(body).is_empty());
}

/// Returns the welcome of player 3 to a world whose ground is `explored`.
fn welcome(explored: Vec<(BodyId, Vec<Run>)>) -> ServerMsg {
    ServerMsg::Welcome(Welcome {
        you: PlayerId(3),
        seed: 3,
        home: crate::universe::HOME_RULE,
        clock: 0.0,
        changes: Changes::default(),
        atlas: String::new(),
        record: None,
        players: Vec::new(),
        explored,
        waypoints: Vec::new(),
    })
}

/// The ground of a welcome is merged at once, and merged again into the world the welcome
/// starts once it is in play.
#[test]
fn the_welcome_ground_outlives_the_new_world() {
    let mut app = alone();
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    app.world_mut()
        .insert_resource(Link::over(Source::Script(script), Role::Host));
    app.update();
    let body = Universe::new(3).home();
    let runs = vec![Run {
        gap: key(3, 10, 7),
        len: 4,
    }];
    let words_of = |app: &App| {
        app.world()
            .resource::<Explored>()
            .leaves(body)
            .map(Leaves::runs)
    };
    app.world_mut()
        .write_message(FromServer(welcome(vec![(body, runs.clone())])));
    app.update();
    assert_eq!(words_of(&app), Some(runs.clone()));
    app.world_mut().insert_resource(Explored::default());
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
    app.update();
    assert_eq!(words_of(&app), Some(runs));
}
