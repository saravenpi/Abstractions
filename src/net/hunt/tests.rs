//! Tests of the hunt of a session: what the server tells goes where the game plays it.

use abstractions_net::ids::BodyId;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{Blow, Event, Rock, ServerMsg};
use bevy::prelude::*;

use super::take::receive;
use super::{Echoes, Hunted};
use crate::net::link::FromServer;

/// Returns an app that takes the hunting messages of the server.
fn app() -> App {
    let mut app = App::new();
    app.add_message::<FromServer>()
        .init_resource::<Hunted>()
        .init_resource::<Echoes>()
        .add_systems(Update, receive);
    app
}

/// Returns a piece of a rock.
fn piece() -> Rock {
    Rock {
        planet: BodyId::default(),
        id: 4,
        grams: 250,
        whole: 900,
    }
}

/// A blow and a rock go to the echoes at once, a message that is not about hunting is ignored, and
/// the rest waits for the world.
#[test]
fn echoes_wait_for_no_world() {
    let mut app = app();
    let blow = Blow {
        victim: PlayerId(2),
        at: [0.0; 3],
        joules: 5.0,
        cut: false,
        dir: [0.0, 0.0, 1.0],
        momentum: 1.0,
    };
    let rock = piece();
    for event in [
        Event::Blow {
            by: PlayerId(1),
            blow,
        },
        Event::Rock(rock),
    ] {
        app.world_mut()
            .write_message(FromServer(ServerMsg::Hunt(event)));
    }
    app.world_mut()
        .write_message(FromServer(ServerMsg::Left(PlayerId(1))));
    app.update();
    let echoes = app.world().resource::<Echoes>();
    assert_eq!(echoes.blows, [(PlayerId(1), blow)]);
    assert_eq!(echoes.rocks, [rock]);
    assert!(app.world().resource::<Hunted>().inbox.is_empty());
}
