//! Tests of the sharing of the explored ground: the size of what goes out and the cells kept
//! with no server.

use std::f64::consts::PI;

use abstractions_net::wire::{CLIENT_MESSAGE_MAX, ClientMsg, encode};
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::{CELLS_MAX, messages};
use crate::map::explored::Explored;
use crate::screens::Screen;
use crate::universe::Universe;

/// A whole body uncovered at once goes out as many messages, each one small enough for the
/// server to read, and every cell once.
#[test]
fn a_big_send_is_cut_under_the_cap() {
    let body = Universe::new(3).home();
    let mut explored = Explored::default();
    explored.reveal(body, DVec3::Y, PI);
    let sent = messages(&mut explored);
    let mut cells = 0;
    for msg in &sent {
        let size = encode(msg).expect("encodes").len();
        assert!(size < CLIENT_MESSAGE_MAX, "{size} bytes");
        let ClientMsg::Explored { cells: some, .. } = msg else {
            panic!("not explored cells: {msg:?}");
        };
        assert!(some.len() <= CELLS_MAX);
        cells += some.len();
    }
    assert!(cells >= 50_000, "{cells} cells");
    assert!(sent.len() > 1);
    assert!(messages(&mut explored).is_empty());
}

/// Returns a headless app with the sharing of the explored ground and no server.
fn alone() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Screen>()
        .init_resource::<Explored>()
        .add_plugins(super::plugin);
    app.update();
    app
}

/// With no server the cells uncovered here are not kept for one.
#[test]
fn nothing_piles_up_offline() {
    let mut app = alone();
    let body = Universe::new(3).home();
    app.world_mut()
        .resource_mut::<Explored>()
        .reveal(body, DVec3::X, 0.01);
    app.update();
    let mut explored = app.world_mut().resource_mut::<Explored>();
    assert!(
        explored
            .words(body)
            .is_some_and(|words| words.iter().any(|w| *w != 0))
    );
    assert!(explored.take_new(body).is_empty());
}
