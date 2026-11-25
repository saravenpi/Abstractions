//! Tests of the one seat a player key holds: a second join takes the seat over, the older
//! connection can no longer write, and the record is kept on a leave and on a silence.

use super::Room;
use super::kit::{hello, join, messages, place_text, room};
use crate::host::{ConnId, Host, Op, Outbox};
use crate::log::PlayerId;
use crate::wire::{ClientMsg, Hello, PlayerInfo, PlayerRecord, Reason, ServerMsg, Welcome};

/// Returns the welcome queued for `conn`, failing the test when there is none.
fn welcome_of(out: &Outbox, conn: ConnId) -> Welcome {
    let found = messages(out, conn).into_iter().find_map(|msg| match msg {
        ServerMsg::Welcome(welcome) => Some(welcome),
        _ => None,
    });
    found.expect("a welcome")
}

/// Returns a record whose place is `altitude` metres up and whose ship is `ship`.
fn record(altitude: &str, ship: &str) -> PlayerRecord {
    PlayerRecord {
        place: place_text(altitude),
        ship: ship.to_string(),
        marks: String::new(),
        pack: String::new(),
    }
}

/// Returns the place the room keeps under the key filled with `key`.
fn kept(room: &Room, key: u8) -> String {
    let world = room.snapshot(0.0);
    world.players[&format!("{key:02x}").repeat(32)]
        .place
        .clone()
}

/// A second join under the same key closes the first connection and takes its seat and its id,
/// even in a full room, so a key never holds two seats. The others hear a leave, then a join.
#[test]
fn a_second_join_of_a_key_replaces_the_first() {
    let mut room = room(2, None);
    let mut out = Outbox::default();
    let old = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    out.clear();
    let new = join(&mut room, 3, hello("Ada", 1), 105.0, &mut out);
    assert_eq!(welcome_of(&out, new).you, PlayerId(1));
    assert!(out.ops().contains(&Op::Close(old, Reason::Replaced)));
    assert!(messages(&out, old).contains(&ServerMsg::Kick(Reason::Replaced)));
    let ada = PlayerInfo {
        id: PlayerId(1),
        name: "Ada".to_string(),
    };
    let heard = messages(&out, bob);
    assert_eq!(
        heard,
        [ServerMsg::Left(PlayerId(1)), ServerMsg::Joined(ada)]
    );
    let names: Vec<String> = room.players().into_iter().map(|p| p.name).collect();
    assert_eq!(names, ["Ada", "Bob"]);
}

/// Once a newer session holds the seat, nothing the older connection sends is kept: not its
/// record, not its goodbye. The record kept is the newest of the newest session.
#[test]
fn a_replaced_session_cannot_write() {
    let mut room = room(4, None);
    let mut out = Outbox::default();
    let old = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let stale = ClientMsg::Record(record("6560.18", "old"));
    room.message(old, stale.clone(), 101.0, &mut out);
    let new = join(&mut room, 2, hello("Ada", 1), 102.0, &mut out);
    let fresh = record("2.0", "new");
    room.message(new, ClientMsg::Record(fresh.clone()), 103.0, &mut out);
    out.clear();
    room.message(old, stale, 104.0, &mut out);
    room.message(old, ClientMsg::Bye, 104.0, &mut out);
    room.closed(old, 105.0, &mut out);
    assert!(out.ops().is_empty());
    assert_eq!(kept(&room, 1), fresh.place);
    assert_eq!(room.players().len(), 1);
    assert_eq!(welcome_of_again(&mut room, 3).record, Some(fresh));
}

/// Joins under key 1 on connection `n` and returns the welcome.
fn welcome_of_again(room: &mut Room, n: u64) -> Welcome {
    let mut out = Outbox::default();
    let conn = join(room, n, hello("Ada", 1), 110.0, &mut out);
    welcome_of(&out, conn)
}

/// A hello that is refused does not send the seated player of the same key away.
#[test]
fn a_refused_join_replaces_nobody() {
    let mut room = room(4, Some("sesame"));
    let mut out = Outbox::default();
    let good = Hello {
        password: Some("sesame".to_string()),
        ..hello("Ada", 1)
    };
    let ada = join(&mut room, 1, good, 100.0, &mut out);
    out.clear();
    let thief = join(&mut room, 2, hello("Ada", 1), 101.0, &mut out);
    assert!(out.ops().contains(&Op::Close(thief, Reason::WrongPassword)));
    assert!(
        !out.ops()
            .iter()
            .any(|op| matches!(op, Op::Close(conn, _) if *conn == ada))
    );
    assert_eq!(room.players().len(), 1);
}

/// The record a player sent last is kept when the connection is lost without a goodbye and when
/// the player goes silent: a joined player not heard for a minute leaves, a heard one stays, and
/// a datagram counts as hearing a player.
#[test]
fn the_place_is_kept_on_a_lost_connection_and_on_silence() {
    let mut room = room(4, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let bob = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    room.saved();
    room.message(ada, ClientMsg::Record(record("3.5", "a")), 101.0, &mut out);
    room.closed(ada, 102.0, &mut out);
    assert!(room.changed());
    assert_eq!(kept(&room, 1), place_text("3.5"));
    room.message(bob, ClientMsg::Record(record("4.5", "b")), 103.0, &mut out);
    out.clear();
    room.tick(162.9, &mut out);
    assert!(!out.ops().iter().any(|op| matches!(op, Op::Close(..))));
    room.tick(163.0, &mut out);
    assert!(out.ops().contains(&Op::Close(bob, Reason::Silent)));
    assert!(room.players().is_empty());
    assert_eq!(kept(&room, 2), place_text("4.5"));
    let cid = join(&mut room, 3, hello("Cid", 3), 200.0, &mut out);
    room.datagram(cid, &[], 250.0, &mut out);
    out.clear();
    room.tick(300.0, &mut out);
    assert!(!out.ops().iter().any(|op| matches!(op, Op::Close(..))));
    assert_eq!(room.players().len(), 1);
}
