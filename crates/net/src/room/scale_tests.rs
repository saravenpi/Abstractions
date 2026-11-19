//! Tests of the relay with many players: every record reaches everyone, within the budget.

use super::kit::{batch, heads, hello, join, room, star};
use crate::host::{DATAGRAM_BUDGET, Host, Op, Outbox};
use crate::wire::Kind;

/// Returns the records player `me` of a hundred is owed: the robot and the ship of each
/// other player, by player.
fn owed(me: u32) -> Vec<(u32, u8)> {
    let others = (1..=100).filter(|player| *player != me);
    let kinds = [Kind::Robot as u8, Kind::Ship as u8];
    others
        .flat_map(|player| kinds.map(|kind| (player, kind)))
        .collect()
}

/// A hundred players around one star each get every new record of the 99 others in one tick,
/// over several datagrams that each fit the budget.
#[test]
fn a_hundred_players_get_every_record() {
    let mut room = room(100, None);
    let mut out = Outbox::default();
    let conns: Vec<_> = (1..=100)
        .map(|n| join(&mut room, n, hello("P", n as u8), 100.0, &mut out))
        .collect();
    for (player, conn) in (1..).zip(&conns) {
        let mut bytes = batch(player, Kind::Robot, star(0), 1.0, 40);
        bytes.extend(batch(player, Kind::Ship, star(0), 1.0, 40));
        room.datagram(*conn, &bytes, 100.0, &mut out);
    }
    out.clear();
    room.tick(100.0, &mut out);
    for (me, conn) in (1..).zip(&conns) {
        let mut got: Vec<(u32, u8)> = heads(&out, *conn)
            .iter()
            .map(|head| (head.player.0, head.kind as u8))
            .collect();
        got.sort_unstable();
        assert_eq!(got, owed(me), "player {me}");
    }
    let datagrams = out.ops().iter().filter_map(|op| match op {
        Op::Datagram(to, range) => Some((*to, range.len())),
        _ => None,
    });
    let mut first = 0;
    for (to, size) in datagrams {
        assert!(size <= DATAGRAM_BUDGET, "{size} bytes");
        first += usize::from(to == conns[0]);
    }
    assert!(first > 1, "{first} datagrams");
}
