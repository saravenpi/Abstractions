//! Tests of joining: the welcome, every refusal, the status, and connections that never join.

use std::net::SocketAddr;

use super::kit::{hello, join, messages, place_text, room};
use crate::host::{ConnId, Host, Op, Outbox};
use crate::log::PlayerId;
use crate::wire::{ClientMsg, PROTOCOL, PlayerInfo, PlayerRecord, Reason, ServerMsg};

/// Returns the welcome queued for `conn`, failing the test when there is none.
fn welcome_of(out: &Outbox, conn: ConnId) -> crate::wire::Welcome {
    let found = messages(out, conn).into_iter().find_map(|msg| match msg {
        ServerMsg::Welcome(welcome) => Some(welcome),
        _ => None,
    });
    found.expect("a welcome")
}

/// A first visit gets the seed, the clock, the atlas, no record and nobody else.
#[test]
fn welcome_carries_the_world() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("  Ada\u{7}  ", 1), 100.0, &mut out);
    let first = welcome_of(&out, ada);
    assert_eq!(
        (first.you, first.seed, first.clock),
        (PlayerId(1), 77, 5_000.0)
    );
    assert_eq!(
        (first.atlas.as_str(), first.record),
        ("bodies = []\n", None)
    );
    assert!(first.players.is_empty());
    assert_eq!(room.players()[0].name, "Ada");
}

/// A player who comes back keeps their id and gets their record and the others, who hear
/// that they joined. A player with no name is called after their id.
#[test]
fn welcome_back_carries_the_record() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let record = PlayerRecord {
        place: place_text("2.0"),
        ship: "s".to_string(),
        marks: "m".to_string(),
        pack: "p".to_string(),
    };
    room.message(ada, ClientMsg::Record(record.clone()), 101.0, &mut out);
    room.message(ada, ClientMsg::Bye, 101.0, &mut out);
    out.clear();
    let bob = join(&mut room, 2, hello("", 2), 102.0, &mut out);
    let ada = join(&mut room, 3, hello("Ada", 1), 110.0, &mut out);
    let back = welcome_of(&out, ada);
    assert_eq!(
        (back.you, back.clock, back.record),
        (PlayerId(1), 5_010.0, Some(record))
    );
    let robot = PlayerInfo {
        id: PlayerId(2),
        name: "Robot 2".to_string(),
    };
    assert_eq!(back.players, vec![robot]);
    let joined = PlayerInfo {
        id: PlayerId(1),
        name: "Ada".to_string(),
    };
    assert!(messages(&out, bob).contains(&ServerMsg::Joined(joined)));
}

/// An older or newer protocol and a wrong password are refused, and the connection closed.
#[test]
fn refuses_protocol_and_password() {
    let mut room = room(16, Some("sesame"));
    let mut out = Outbox::default();
    let cases = [
        (PROTOCOL - 1, Some("sesame"), Reason::Outdated),
        (PROTOCOL + 1, None, Reason::Newer),
    ];
    let wrong = [
        (PROTOCOL, Some("sesamE"), Reason::WrongPassword),
        (PROTOCOL, None, Reason::WrongPassword),
    ];
    for (n, (protocol, password, reason)) in (1..).zip(cases.into_iter().chain(wrong)) {
        let mut asked = hello("Eve", 9);
        asked.protocol = protocol;
        asked.password = password.map(str::to_string);
        let conn = join(&mut room, n, asked, 100.0, &mut out);
        assert_eq!(messages(&out, conn), vec![ServerMsg::Refused(reason)]);
        assert!(out.ops().contains(&Op::Close(conn, reason)));
    }
    let mut right = hello("Ada", 1);
    right.password = Some("sesame".to_string());
    let ada = join(&mut room, 9, right, 100.0, &mut out);
    assert_eq!(room.players().len(), 1);
    assert!(matches!(messages(&out, ada)[0], ServerMsg::Welcome(_)));
}

/// A full room refuses the next player.
#[test]
fn refuses_when_full() {
    let mut room = room(2, None);
    let mut out = Outbox::default();
    join(&mut room, 1, hello("A", 1), 100.0, &mut out);
    join(&mut room, 2, hello("B", 2), 100.0, &mut out);
    let third = join(&mut room, 3, hello("C", 3), 100.0, &mut out);
    assert_eq!(
        messages(&out, third),
        vec![ServerMsg::Refused(Reason::Full)]
    );
    assert_eq!(room.players().len(), 2);
}

/// The status is answered before any hello, and a second hello is ignored.
#[test]
fn status_before_hello_and_one_hello() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let conn = ConnId(1);
    room.connected(conn, "10.0.0.1:1".parse().unwrap(), 100.0, &mut out);
    room.message(conn, ClientMsg::Status, 100.0, &mut out);
    let ServerMsg::Status(status) = &messages(&out, conn)[0] else {
        panic!("not a status");
    };
    assert_eq!(
        (status.players, status.cap, status.protocol),
        (0, 16, PROTOCOL)
    );
    assert_eq!(
        (status.name.as_str(), status.version.as_str()),
        ("Test", "0.9.0")
    );
    room.message(conn, ClientMsg::Hello(hello("A", 1)), 100.0, &mut out);
    room.message(conn, ClientMsg::Hello(hello("B", 2)), 100.0, &mut out);
    assert_eq!(messages(&out, conn).len(), 2);
    assert_eq!(room.players()[0].name, "A");
}

/// A connection that only asks for the status is closed after ten seconds; a joined one stays.
#[test]
fn closes_connections_that_never_join() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let idle = ConnId(1);
    room.connected(idle, "10.0.0.1:1".parse().unwrap(), 100.0, &mut out);
    let ada = join(&mut room, 2, hello("Ada", 2), 100.0, &mut out);
    room.message(idle, ClientMsg::Status, 105.0, &mut out);
    room.tick(109.9, &mut out);
    assert!(!out.ops().iter().any(|op| matches!(op, Op::Close(..))));
    room.tick(110.0, &mut out);
    assert!(out.ops().contains(&Op::Close(idle, Reason::Closing)));
    assert!(!out.ops().contains(&Op::Close(ada, Reason::Closing)));
    assert!(room.busy());
}

/// The ninth connection from one address is closed at once.
#[test]
fn limits_connections_per_address() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let addr: SocketAddr = "10.0.0.1:1".parse().unwrap();
    for n in 0..9 {
        room.connected(ConnId(n), addr, 100.0, &mut out);
    }
    assert_eq!(out.ops(), [Op::Close(ConnId(8), Reason::Full)]);
}
