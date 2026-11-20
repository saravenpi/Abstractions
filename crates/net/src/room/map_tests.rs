//! Tests of the waypoints the room shares: one per player.

use super::Room;
use super::kit::{hello, join, messages, room};
use crate::host::{ConnId, Host, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{ClientMsg, ServerMsg, Waypoint, Welcome};

/// Returns a room with Ada and Bob in it, their connections, and an empty outbox.
pub(super) fn pair() -> (Room, ConnId, ConnId, Outbox) {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    out.clear();
    (room, ada, bob, out)
}

/// Lets Cyd in at network time `now` and returns her welcome.
pub(super) fn welcome_of_cyd(room: &mut Room, now: f64) -> Welcome {
    let mut out = Outbox::default();
    let cyd = join(room, 3, hello("Cyd", 3), now, &mut out);
    match messages(&out, cyd).first() {
        Some(ServerMsg::Welcome(welcome)) => welcome.clone(),
        other => panic!("no welcome but {other:?}"),
    }
}

/// Returns a waypoint on the default body named `name`.
fn mark(name: &str) -> Waypoint {
    Waypoint {
        body: BodyId::default(),
        dir: [0.0, 2.0, 0.0],
        name: name.to_string(),
    }
}

/// A waypoint goes to the others with a unit direction and a clean name, a flat one is
/// ignored, and a later player finds it in the welcome.
#[test]
fn a_waypoint_reaches_the_others() {
    let (mut room, ada, bob, mut out) = pair();
    room.message(
        ada,
        ClientMsg::Waypoint(Some(mark(" Camp\n"))),
        100.0,
        &mut out,
    );
    let shown = Waypoint {
        dir: [0.0, 1.0, 0.0],
        ..mark("Camp")
    };
    let sent = ServerMsg::Waypoint {
        player: PlayerId(1),
        waypoint: Some(shown.clone()),
    };
    assert_eq!(messages(&out, bob), [sent]);
    assert!(messages(&out, ada).is_empty());
    let flat = Waypoint {
        dir: [0.0; 3],
        ..mark("Nowhere")
    };
    room.message(ada, ClientMsg::Waypoint(Some(flat)), 100.1, &mut out);
    let welcome = welcome_of_cyd(&mut room, 100.2);
    assert_eq!(welcome.waypoints, [(PlayerId(1), shown)]);
}

/// A player who leaves has their waypoint cleared for everyone, before the news they left.
#[test]
fn a_waypoint_is_cleared_on_leave() {
    let (mut room, ada, bob, mut out) = pair();
    room.message(
        ada,
        ClientMsg::Waypoint(Some(mark("Camp"))),
        100.0,
        &mut out,
    );
    out.clear();
    room.closed(ada, 101.0, &mut out);
    let cleared = ServerMsg::Waypoint {
        player: PlayerId(1),
        waypoint: None,
    };
    assert_eq!(messages(&out, bob), [cleared, ServerMsg::Left(PlayerId(1))]);
}
