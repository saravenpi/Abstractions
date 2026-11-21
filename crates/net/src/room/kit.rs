//! Helpers the room tests share: a room, players who join, and what the outbox holds.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use glam::IVec3;

use super::{Room, RoomConfig};
use crate::host::{ConnId, Host, Op, Outbox};
use crate::ids::{BodyId, StarId};
use crate::log::PlayerId;
use crate::store::World;
use crate::wire::{ClientMsg, Frame, Head, Hello, Kind, PROTOCOL, Record, ServerMsg, push};

/// Returns a room of `cap` seats with `password`, at network time 100.
pub(super) fn room(cap: u16, password: Option<&str>) -> Room {
    let config = RoomConfig {
        name: "Test".to_string(),
        password: password.map(str::to_string),
        cap,
        version: "0.9.0".to_string(),
    };
    let world = World {
        seed: 77,
        clock: 5_000.0,
        atlas: "bodies = []\n".to_string(),
        ..World::default()
    };
    Room::new(config, world, 100.0)
}

/// Returns the hello of a player named `name` with a key filled with `key`.
pub(super) fn hello(name: &str, key: u8) -> Hello {
    Hello {
        protocol: PROTOCOL,
        name: name.to_string(),
        key: [key; 32],
        password: None,
    }
}

/// Opens connection `n` from its own address and sends `hello` on it at network time `now`.
pub(super) fn join(room: &mut Room, n: u64, hello: Hello, now: f64, out: &mut Outbox) -> ConnId {
    let conn = ConnId(n);
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, n as u8)), 24650);
    room.connected(conn, addr, now, out);
    room.message(conn, ClientMsg::Hello(hello), now, out);
    conn
}

/// Returns the messages queued for `conn`, oldest first.
pub(super) fn messages(out: &Outbox, conn: ConnId) -> Vec<ServerMsg> {
    let sent = out.ops().iter().filter_map(|op| match op {
        Op::Send(to, msg) if *to == conn => Some(msg.clone()),
        _ => None,
    });
    sent.collect()
}

/// Returns the heads of every record in the datagrams queued for `conn`.
pub(super) fn heads(out: &Outbox, conn: ConnId) -> Vec<Head> {
    let ranges = out.ops().iter().filter_map(|op| match op {
        Op::Datagram(to, range) if *to == conn => Some(range.clone()),
        _ => None,
    });
    ranges
        .flat_map(|range| {
            crate::wire::records(out.bytes(range))
                .map(|record| record.head)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Returns the text of a place as the game writes it: `altitude` metres above the sea level of
/// the first planet of a star, straight up its axis. `altitude` is written as given, so `nan`
/// and `inf` make a damaged place.
pub(super) fn place_text(altitude: &str) -> String {
    format!(
        "dir = [0.0, 1.0, 0.0]\naltitude = {altitude}\n\n[body]\nindex = 0\nmoon = 0\n\n\
         [body.star]\ngalaxy = [0, 0, 0]\nsector = [35, 0, -44]\nindex = 0\n"
    )
}

/// Returns the star with rank `index` in the home sector.
pub(super) fn star(index: u8) -> StarId {
    StarId {
        index,
        sector: IVec3::ZERO,
        ..StarId::default()
    }
}

/// Returns a batch holding one record of `kind` for `player` on a planet of `star`, sampled at
/// `time`, with a body of `size` bytes.
pub(super) fn batch(player: u32, kind: Kind, star: StarId, time: f64, size: usize) -> Vec<u8> {
    let head = Head {
        player: PlayerId(player),
        kind,
        frame: Frame::Surface(BodyId {
            star,
            index: 1,
            moon: 0,
        }),
        time,
        flags: 0,
    };
    let body = vec![7; size];
    let mut bytes = Vec::new();
    push(&mut bytes, &Record { head, body: &body });
    bytes
}
