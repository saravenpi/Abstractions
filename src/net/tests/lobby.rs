//! The rows of the multiplayer menu the network fills: the status of a saved server and a
//! game heard on the local network.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use abstractions_net::io::CloseReason;
use abstractions_net::lan::{Beacon, Found};
use abstractions_net::wire::{PROTOCOL, Status};

use crate::net::RowStatus;
use crate::net::scan::lan::row_of;
use crate::net::scan::status::row_status;

/// Returns the status of a server speaking `protocol`, with 3 of 16 players.
fn status(protocol: u16) -> Status {
    Status {
        name: "Space".to_string(),
        players: 3,
        cap: 16,
        version: "0.9.0".to_string(),
        protocol,
    }
}

/// Returns the row status of `status` answered in 42 ms with the key `[1; 32]`, when the
/// player remembers `known` for the address.
fn answered(status: Status, known: Option<[u8; 32]>) -> (String, RowStatus) {
    let rtt = Duration::from_millis(42);
    row_status(&Ok((status, rtt, [1; 32])), known)
}

/// A server of this protocol is compatible, an older one is not, and both show their
/// players, round trip and version.
#[test]
fn a_status_shows_whether_the_server_is_compatible() {
    let online = |compatible| RowStatus::Online {
        players: 3,
        cap: 16,
        ping_ms: 42,
        version: "0.9.0".to_string(),
        compatible,
    };
    let same = answered(status(PROTOCOL), Some([1; 32]));
    assert_eq!(same, ("Space".to_string(), online(true)));
    assert_eq!(answered(status(PROTOCOL - 1), None).1, online(false));
    assert_eq!(answered(status(PROTOCOL + 1), None).1, online(false));
}

/// A server showing another key than the remembered one shows as changed; one that does not
/// answer as offline.
#[test]
fn a_changed_key_and_silence_show_in_the_row() {
    assert_eq!(
        answered(status(PROTOCOL), Some([2; 32])).1,
        RowStatus::KeyChanged
    );
    let silent = row_status(&Err(CloseReason::Timeout), None);
    assert_eq!(silent.1, RowStatus::Offline);
}

/// A game heard on the network is joined at the sender's address and the beacon's port, with
/// the key the beacon announced.
#[test]
fn a_beacon_makes_a_row_with_its_key() {
    let found = Found {
        addr: SocketAddr::from(([192, 168, 1, 20], 24652)),
        beacon: Beacon {
            protocol: PROTOCOL,
            version: "0.9.0".to_string(),
            name: "Ada's world".to_string(),
            players: 2,
            cap: 16,
            port: 24652,
            key: [9; 32],
        },
        seen: Instant::now(),
    };
    let row = row_of(&found);
    assert_eq!(row.address, "192.168.1.20:24652");
    assert_eq!((row.name.as_str(), row.key), ("Ada's world", Some([9; 32])));
    assert!(matches!(
        row.status,
        RowStatus::Online {
            players: 2,
            compatible: true,
            ..
        }
    ));
}
