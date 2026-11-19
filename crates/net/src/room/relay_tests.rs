//! Tests of the state relay: who gets a record, how often, and what is dropped.

use super::kit::{batch, heads, hello, join, room, star};
use crate::host::{DATAGRAM_BUDGET, Host, Op, Outbox};
use crate::log::PlayerId;
use crate::wire::Kind;

/// Returns a room with Ada (player 1) and Bob (player 2) in it, and their connections.
fn pair() -> (super::Room, crate::host::ConnId, crate::host::ConnId) {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    (room, ada, bob)
}

/// Returns [`pair`] with Bob seen around the home star.
fn near_pair() -> (super::Room, crate::host::ConnId, crate::host::ConnId) {
    let (mut room, ada, bob) = pair();
    let mut out = Outbox::default();
    let bytes = batch(2, Kind::Ship, star(0), 0.0, 40);
    room.datagram(bob, &bytes, 100.0, &mut out);
    (room, ada, bob)
}

/// A record of Ada reaches Bob once and never Ada herself.
#[test]
fn record_reaches_the_others_only() {
    let (mut room, ada, bob) = pair();
    let mut out = Outbox::default();
    room.datagram(
        ada,
        &batch(1, Kind::Robot, star(0), 1.0, 40),
        100.1,
        &mut out,
    );
    room.tick(100.1, &mut out);
    let got = heads(&out, bob);
    assert_eq!(got.len(), 1);
    assert_eq!((got[0].player, got[0].kind), (PlayerId(1), Kind::Robot));
    assert!(heads(&out, ada).is_empty());
    out.clear();
    room.tick(100.2, &mut out);
    assert!(heads(&out, bob).is_empty());
}

/// Around another star, a player gets the others' records at most once a second.
#[test]
fn other_star_gets_one_record_a_second() {
    let (mut room, ada, bob) = pair();
    let mut out = Outbox::default();
    room.datagram(
        bob,
        &batch(2, Kind::Robot, star(9), 0.0, 40),
        100.0,
        &mut out,
    );
    let mut got = 0;
    for tick in 0..60 {
        let now = 100.0 + f64::from(tick) / 30.0;
        room.datagram(ada, &batch(1, Kind::Robot, star(0), now, 40), now, &mut out);
        room.tick(now, &mut out);
        got += heads(&out, bob).len();
        out.clear();
    }
    assert_eq!(got, 2);
    room.datagram(
        bob,
        &batch(2, Kind::Robot, star(0), 2.0, 40),
        102.0,
        &mut out,
    );
    for tick in 0..3 {
        let now = 102.0 + f64::from(tick) / 30.0;
        room.datagram(ada, &batch(1, Kind::Robot, star(0), now, 40), now, &mut out);
        room.tick(now, &mut out);
    }
    assert_eq!(heads(&out, bob).len(), 3);
}

/// A record that claims another player's id, a body over 128 bytes and an older sample are
/// dropped, while two records of one kind in a batch both go out, in order.
#[test]
fn drops_spoofed_large_old_and_repeated_records() {
    let (mut room, ada, bob) = near_pair();
    let mut out = Outbox::default();
    room.datagram(
        ada,
        &batch(2, Kind::Robot, star(0), 1.0, 40),
        100.0,
        &mut out,
    );
    room.datagram(
        ada,
        &batch(1, Kind::Ship, star(0), 1.0, 129),
        100.0,
        &mut out,
    );
    room.tick(100.0, &mut out);
    assert!(heads(&out, bob).is_empty());
    let mut two = batch(1, Kind::Robot, star(0), 5.0, 40);
    two.extend(batch(1, Kind::Robot, star(0), 6.0, 40));
    room.datagram(ada, &two, 100.1, &mut out);
    room.datagram(
        ada,
        &batch(1, Kind::Robot, star(0), 4.0, 40),
        100.1,
        &mut out,
    );
    room.tick(100.1, &mut out);
    let times: Vec<f64> = heads(&out, bob).iter().map(|head| head.time).collect();
    assert_eq!(times, [5.0, 6.0]);
}

/// Of six records of one kind between two ticks, the last four go out in order, and nothing is
/// sent again on the next tick.
#[test]
fn the_last_four_records_go_out_in_order() {
    let (mut room, ada, bob) = near_pair();
    let mut out = Outbox::default();
    for step in 0..6 {
        let time = 1.0 + f64::from(step) / 30.0;
        let bytes = batch(1, Kind::Robot, star(0), time, 40);
        room.datagram(ada, &bytes, 100.0 + time, &mut out);
    }
    room.tick(101.2, &mut out);
    let times: Vec<f64> = heads(&out, bob).iter().map(|head| head.time).collect();
    let want: Vec<f64> = (2..6).map(|step| 1.0 + f64::from(step) / 30.0).collect();
    assert_eq!(times, want);
    out.clear();
    room.datagram(
        ada,
        &batch(1, Kind::Robot, star(0), 2.0, 40),
        101.3,
        &mut out,
    );
    room.tick(101.3, &mut out);
    let times: Vec<f64> = heads(&out, bob).iter().map(|head| head.time).collect();
    assert_eq!(times, [2.0]);
}

/// Many records split into datagrams that each fit the budget.
#[test]
fn batches_fit_the_budget() {
    let mut room = room(16, None);
    let mut out = Outbox::default();
    let conns: Vec<_> = (1..=16)
        .map(|n| join(&mut room, n, hello("P", n as u8), 100.0, &mut out))
        .collect();
    for (player, conn) in (1..).zip(&conns) {
        let mut bytes = batch(player, Kind::Robot, star(0), 1.0, 128);
        bytes.extend(batch(player, Kind::Ship, star(0), 1.0, 128));
        room.datagram(*conn, &bytes, 100.0, &mut out);
    }
    out.clear();
    room.tick(100.0, &mut out);
    assert_eq!(heads(&out, conns[0]).len(), 30);
    for op in out.ops() {
        if let Op::Datagram(_, range) = op {
            assert!(range.len() <= DATAGRAM_BUDGET, "{} bytes", range.len());
        }
    }
}
