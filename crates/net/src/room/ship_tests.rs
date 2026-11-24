//! Tests of shared ship ownership, pilot refusal, authority handoff, parked heartbeats and disconnects.

use super::kit::{batch, heads, hello, join, messages, room, star};
use crate::host::{Host, Outbox};
use crate::log::PlayerId;
use crate::wire::{ClientMsg, Kind, ServerMsg, ShipAct, ShipEvent, ShipPilot};

/// Sends one `Kind::Ship` datagram on behalf of `conn` stamped for ship `ship_id`.
fn send_ship(
    room: &mut super::Room,
    conn: crate::host::ConnId,
    ship_id: u32,
    now: f64,
    out: &mut Outbox,
) {
    let bytes = batch(ship_id, Kind::Ship, star(0), now, 41);
    room.datagram(conn, &bytes, now, out);
}

/// Boarding another player's ship grants authority, relays records back to owner and blocks rivals.
#[test]
fn board_hands_off_authority_and_blocks_second_pilot() {
    let mut room = room(8, None);
    let mut out = Outbox::default();
    let c1 = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let c2 = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    let c3 = join(&mut room, 3, hello("Cy", 3), 100.0, &mut out);
    send_ship(&mut room, c1, 1, 100.5, &mut out);
    out.clear();
    room.message(
        c2,
        ClientMsg::Ship(ShipAct::Board(PlayerId(1))),
        101.0,
        &mut out,
    );
    let boarded = ServerMsg::Ship(ShipEvent::Boarded {
        ship: PlayerId(1),
        pilot: PlayerId(2),
    });
    assert!(messages(&out, c1).contains(&boarded));
    assert!(messages(&out, c2).contains(&boarded));
    out.clear();
    room.message(
        c3,
        ClientMsg::Ship(ShipAct::Board(PlayerId(1))),
        101.1,
        &mut out,
    );
    assert_eq!(
        messages(&out, c3),
        vec![ServerMsg::Ship(ShipEvent::Denied(PlayerId(1)))]
    );
    out.clear();
    send_ship(&mut room, c1, 1, 101.2, &mut out);
    send_ship(&mut room, c2, 1, 101.2, &mut out);
    room.tick(101.2, &mut out);
    let h1 = heads(&out, c1);
    assert!(
        h1.iter()
            .any(|h| h.kind == Kind::Ship && h.player == PlayerId(1))
    );
}

/// Switching to another ship puts the player's parked ship on 2-second server heartbeats, and an
/// owner leaving keeps their ship alive while another player still pilots it.
#[test]
fn parked_heartbeats_and_disconnect_while_piloted() {
    let mut room = room(8, None);
    let mut out = Outbox::default();
    let c1 = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let c2 = join(&mut room, 2, hello("Bob", 2), 100.0, &mut out);
    send_ship(&mut room, c1, 1, 100.2, &mut out);
    send_ship(&mut room, c2, 2, 100.2, &mut out);
    room.tick(100.2, &mut out);
    room.message(
        c2,
        ClientMsg::Ship(ShipAct::Board(PlayerId(1))),
        100.5,
        &mut out,
    );
    room.tick(100.5, &mut out);
    out.clear();
    room.tick(103.0, &mut out);
    let h1 = heads(&out, c1);
    assert!(h1.iter().any(|h| h.kind == Kind::Ship
        && h.player == PlayerId(2)
        && (h.time - 5003.0).abs() < 1e-6));
    out.clear();
    room.message(c1, ClientMsg::Bye, 103.5, &mut out);
    assert!(!messages(&out, c2).contains(&ServerMsg::Ship(ShipEvent::Gone(PlayerId(1)))));
    let c3 = join(&mut room, 3, hello("Cy", 3), 104.0, &mut out);
    let world = ServerMsg::Ship(ShipEvent::World(vec![ShipPilot {
        ship: PlayerId(1),
        pilot: PlayerId(2),
    }]));
    assert!(messages(&out, c3).contains(&world));
    out.clear();
    room.message(
        c2,
        ClientMsg::Ship(ShipAct::Leave(PlayerId(1))),
        104.5,
        &mut out,
    );
    assert!(messages(&out, c3).contains(&ServerMsg::Ship(ShipEvent::Gone(PlayerId(1)))));
}
