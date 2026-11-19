//! Tests of the map the room shares: explored cells merged and sent once a second, and one
//! waypoint per player.

use super::Room;
use super::kit::{hello, join, messages, room};
use crate::host::{ConnId, Host, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{ClientMsg, ServerMsg, Waypoint, Welcome};

/// Returns a room with Ada and Bob in it, their connections, and an empty outbox.
fn pair() -> (Room, ConnId, ConnId, Outbox) {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    out.clear();
    (room, ada, bob, out)
}

/// Lets Cyd in at network time `now` and returns her welcome.
fn welcome_of_cyd(room: &mut Room, now: f64) -> Welcome {
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

/// Returns the cells of every explored message in `msgs`, in order.
fn cells(msgs: &[ServerMsg]) -> Vec<Vec<u32>> {
    let explored = msgs.iter().filter_map(|msg| match msg {
        ServerMsg::Explored { cells, .. } => Some(cells.clone()),
        _ => None,
    });
    explored.collect()
}

/// New cells go to everyone at most once a second per body, and known or out-of-grid cells
/// never do.
#[test]
fn explored_cells_merge_and_go_out_once_a_second() {
    let (mut room, ada, bob, mut out) = pair();
    let body = BodyId::default();
    let cells_of = |cells: Vec<u32>| ClientMsg::Explored { body, cells };
    room.message(ada, cells_of(vec![3, 3, 70, 512 * 256]), 100.0, &mut out);
    room.tick(100.0, &mut out);
    assert_eq!(cells(&messages(&out, bob)), [vec![3, 70]]);
    out.clear();
    room.message(bob, cells_of(vec![70, 5]), 100.2, &mut out);
    room.tick(100.5, &mut out);
    assert!(cells(&messages(&out, ada)).is_empty());
    room.tick(101.0, &mut out);
    assert_eq!(cells(&messages(&out, ada)), [vec![5]]);
}

/// A later player finds every explored cell in the welcome, and the snapshot holds them.
#[test]
fn a_later_player_finds_the_explored_ground() {
    let (mut room, ada, _, mut out) = pair();
    let body = BodyId::default();
    let cells = vec![3, 5, 70];
    room.message(ada, ClientMsg::Explored { body, cells }, 100.0, &mut out);
    let welcome = welcome_of_cyd(&mut room, 100.5);
    let (got, bits) = &welcome.explored[0];
    assert_eq!(
        (*got, bits[0], bits[1]),
        (body, (1 << 3) | (1 << 5), 1 << 6)
    );
    assert_eq!(room.snapshot(102.0).explored, welcome.explored);
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
