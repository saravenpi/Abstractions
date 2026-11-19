//! Tests of joined players: the log, pings, records, leaving, flooding and the snapshot.

use super::kit::{hello, join, messages, room};
use crate::host::{Host, Op, Outbox};
use crate::ids::BodyId;
use crate::log::{Change, Entry, PlayerId};
use crate::wire::{ClientMsg, PlayerRecord, Reason, ServerMsg};

/// Returns the body of rank `index` around the default star.
fn body(index: u8) -> BodyId {
    BodyId {
        index,
        ..BodyId::default()
    }
}

/// A visit enters the log once and reaches everyone, the sender included.
#[test]
fn duplicate_visit_is_refused_and_broadcast_once() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    out.clear();
    room.message(
        ada,
        ClientMsg::Change(Change::Visited(body(3))),
        101.0,
        &mut out,
    );
    room.message(
        bob,
        ClientMsg::Change(Change::Visited(body(3))),
        102.0,
        &mut out,
    );
    let entry = Entry {
        player: PlayerId(1),
        time: 5_001.0,
        change: Change::Visited(body(3)),
    };
    for conn in [ada, bob] {
        assert_eq!(messages(&out, conn), vec![ServerMsg::Change(entry.clone())]);
    }
    assert_eq!(room.snapshot(102.0).changes.entries, vec![entry]);
}

/// A player adds at most ten entries a second.
#[test]
fn limits_log_appends() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    for index in 0..15 {
        room.message(
            ada,
            ClientMsg::Change(Change::Visited(body(index))),
            100.5,
            &mut out,
        );
    }
    assert_eq!(room.snapshot(101.0).changes.entries.len(), 10);
    room.message(
        ada,
        ClientMsg::Change(Change::Visited(body(20))),
        101.6,
        &mut out,
    );
    assert_eq!(room.snapshot(102.0).changes.entries.len(), 11);
}

/// A ping is answered at once with the universe time.
#[test]
fn answers_a_ping() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    out.clear();
    room.message(ada, ClientMsg::Ping { id: 4, sent: 1.5 }, 102.5, &mut out);
    let pong = ServerMsg::Pong {
        id: 4,
        sent: 1.5,
        clock: 5_002.5,
    };
    assert_eq!(messages(&out, ada), vec![pong]);
}

/// The snapshot advances the clock and holds the record of a player who left.
#[test]
fn snapshot_keeps_clock_and_records() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 0xab), 100.0, &mut out);
    let record = PlayerRecord {
        place: "here".to_string(),
        ship: "red".to_string(),
        marks: "lake".to_string(),
    };
    room.message(ada, ClientMsg::Record(record), 101.0, &mut out);
    room.closed(ada, 120.0, &mut out);
    let world = room.snapshot(160.0);
    assert_eq!(world.clock, 5_060.0);
    let member = &world.players[&"ab".repeat(32)];
    assert_eq!(
        (
            member.name.as_str(),
            member.place.as_str(),
            member.ship.as_str(),
            member.marks.as_str()
        ),
        ("Ada", "here", "red", "lake")
    );
    assert!(!room.busy());
}

/// Leaving by a bye or by a lost connection tells the others.
#[test]
fn leaving_sends_left() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    let cid = join(&mut room, 3, hello("Cid", 3), 100.0, &mut out);
    out.clear();
    room.message(ada, ClientMsg::Bye, 101.0, &mut out);
    room.closed(bob, 101.0, &mut out);
    assert_eq!(
        messages(&out, cid),
        vec![ServerMsg::Left(PlayerId(1)), ServerMsg::Left(PlayerId(2))]
    );
    assert!(messages(&out, ada).is_empty());
    assert_eq!(room.players().len(), 1);
}

/// A connection that sends more than sixty messages a second is closed and its player leaves.
#[test]
fn flooding_closes_the_connection() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    for id in 0..60 {
        room.message(ada, ClientMsg::Ping { id, sent: 0.0 }, 101.5, &mut out);
    }
    assert!(!out.ops().contains(&Op::Close(ada, Reason::Flooding)));
    room.message(ada, ClientMsg::Ping { id: 60, sent: 0.0 }, 101.5, &mut out);
    assert!(out.ops().contains(&Op::Close(ada, Reason::Flooding)));
    assert!(messages(&out, bob).contains(&ServerMsg::Left(PlayerId(1))));
}
