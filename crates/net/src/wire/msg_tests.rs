//! Tests of the reliable messages: every one survives the wire, and the numbers that must
//! never change have not changed.

use super::body_tests::planet;
use super::{
    ClientMsg, Hello, PlayerInfo, PlayerRecord, Reason, ServerMsg, ShipAct, ShipEvent, ShipPilot,
    Status, Waypoint, Welcome,
};
use super::{PROTOCOL, decode, encode};
use crate::leaves::Run;
use crate::log::{Change, Changes, Entry, PlayerId};

/// Returns a named waypoint on the test planet.
fn waypoint() -> Waypoint {
    Waypoint {
        body: planet(),
        dir: [0.6, 0.0, -0.8],
        name: "Camp".to_string(),
    }
}

/// Returns a welcome with a log, an atlas, a record and another player.
fn welcome() -> Welcome {
    let mut changes = Changes::default();
    changes.push(PlayerId(2), 1234.5, Change::Visited(planet()));
    Welcome {
        you: PlayerId(3),
        seed: 0xdead_beef_cafe,
        home: 1,
        clock: 98_765.432_1,
        changes,
        atlas: "[[seen]]\nindex = 2\n".to_string(),
        record: Some(PlayerRecord {
            place: "kind = \"surface\"".to_string(),
            ship: String::new(),
            marks: "[[mark]]\nname = \"Lake\"\n".to_string(),
        }),
        players: vec![PlayerInfo {
            id: PlayerId(2),
            name: "Zoé".to_string(),
        }],
        explored: vec![(
            planet(),
            vec![
                Run { gap: 7, len: 3 },
                Run {
                    gap: 1 << 40,
                    len: 9,
                },
            ],
        )],
        waypoints: vec![(PlayerId(2), waypoint())],
    }
}

/// Every message of a player comes back as it was sent.
#[test]
fn client_messages_come_back() {
    let hello = Hello {
        protocol: PROTOCOL,
        name: "Yann".to_string(),
        key: [7; 32],
        password: Some("secret".to_string()),
    };
    let all = [
        ClientMsg::Status,
        ClientMsg::Hello(hello),
        ClientMsg::Change(Change::Visited(planet())),
        ClientMsg::Record(PlayerRecord::default()),
        ClientMsg::Ping { id: 9, sent: 1.25 },
        ClientMsg::Bye,
    ];
    for message in all {
        let bytes = encode(&message).unwrap();
        let (got, used) = decode::<ClientMsg>(&bytes, bytes.len()).unwrap().unwrap();
        assert_eq!((got, used), (message, bytes.len()));
    }
}

/// Every message of a server comes back as it was sent.
#[test]
fn server_messages_come_back() {
    let status = Status {
        name: "Home".to_string(),
        players: 3,
        cap: 16,
        version: "0.9.0".to_string(),
        protocol: PROTOCOL,
    };
    let entry = welcome().changes.entries[0].clone();
    let info = welcome().players[0].clone();
    let all = [
        ServerMsg::Status(status),
        ServerMsg::Welcome(welcome()),
        ServerMsg::Refused(Reason::Outdated),
        ServerMsg::Joined(info),
        ServerMsg::Left(PlayerId(2)),
        ServerMsg::Change(Entry { time: 5.0, ..entry }),
        ServerMsg::Pong {
            id: 9,
            sent: 1.25,
            clock: 77.0,
        },
        ServerMsg::Kick(Reason::Closing),
    ];
    for message in all {
        let bytes = encode(&message).unwrap();
        let (got, used) = decode::<ServerMsg>(&bytes, bytes.len()).unwrap().unwrap();
        assert_eq!((got, used), (message, bytes.len()));
    }
}

/// The explored cells and the waypoint of a player come back as they were sent.
#[test]
fn client_map_messages_come_back() {
    let client = [
        ClientMsg::Explored {
            body: planet(),
            runs: vec![Run { gap: 5, len: 2 }, Run { gap: 99, len: 1 }],
        },
        ClientMsg::Waypoint(Some(waypoint())),
        ClientMsg::Waypoint(None),
    ];
    for message in client {
        let bytes = encode(&message).unwrap();
        assert_eq!(
            decode::<ClientMsg>(&bytes, bytes.len()).unwrap().unwrap().0,
            message
        );
    }
}

/// The explored cells and the waypoints a server sends come back as they were sent.
#[test]
fn server_map_messages_come_back() {
    let server = [
        ServerMsg::Explored {
            body: planet(),
            runs: vec![Run { gap: 7, len: 1 }],
        },
        ServerMsg::Waypoint {
            player: PlayerId(2),
            waypoint: Some(waypoint()),
        },
        ServerMsg::Waypoint {
            player: PlayerId(2),
            waypoint: None,
        },
    ];
    for message in server {
        let bytes = encode(&message).unwrap();
        assert_eq!(
            decode::<ServerMsg>(&bytes, bytes.len()).unwrap().unwrap().0,
            message
        );
    }
}

/// The status request and its answer keep number zero, whatever the version: an old game must
/// still be able to ask a new server what it is.
#[test]
fn the_status_messages_keep_their_number() {
    assert_eq!(encode(&ClientMsg::Status).unwrap(), [1, 0, 0, 0, 0]);
    assert_eq!(encode(&ClientMsg::Bye).unwrap(), [1, 0, 0, 0, 5]);
    let status = ServerMsg::Status(Status {
        name: "a".to_string(),
        players: 1,
        cap: 2,
        version: "b".to_string(),
        protocol: 3,
    });
    let bytes = encode(&status).unwrap();
    assert_eq!(bytes[4..], [0, 1, b'a', 1, 2, 1, b'b', 3]);
    assert_eq!(encode(&ServerMsg::Kick(Reason::Full)).unwrap()[4..], [7, 1]);
    assert_eq!(
        encode(&ClientMsg::Waypoint(None)).unwrap(),
        [2, 0, 0, 0, 7, 0]
    );
    let cleared = ServerMsg::Waypoint {
        player: PlayerId(1),
        waypoint: None,
    };
    assert_eq!(encode(&cleared).unwrap()[4..], [9, 1, 0]);
}

/// Shared ship pilot messages come back intact on both ends.
#[test]
fn ship_messages_come_back() {
    let client = [
        ClientMsg::Ship(ShipAct::Board(PlayerId(4))),
        ClientMsg::Ship(ShipAct::Leave(PlayerId(4))),
    ];
    for message in client {
        let bytes = encode(&message).unwrap();
        let got = decode::<ClientMsg>(&bytes, bytes.len()).unwrap().unwrap().0;
        assert_eq!(got, message);
    }
    let server = [
        ServerMsg::Ship(ShipEvent::World(vec![ShipPilot {
            ship: PlayerId(4),
            pilot: PlayerId(2),
        }])),
        ServerMsg::Ship(ShipEvent::Boarded {
            ship: PlayerId(4),
            pilot: PlayerId(2),
        }),
        ServerMsg::Ship(ShipEvent::Left {
            ship: PlayerId(4),
            pilot: PlayerId(2),
        }),
        ServerMsg::Ship(ShipEvent::Denied(PlayerId(4))),
        ServerMsg::Ship(ShipEvent::Gone(PlayerId(4))),
    ];
    for message in server {
        let bytes = encode(&message).unwrap();
        let got = decode::<ServerMsg>(&bytes, bytes.len()).unwrap().unwrap().0;
        assert_eq!(got, message);
    }
}
