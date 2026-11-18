//! Tests of the line `ping` prints.

use std::time::Duration;

use abstractions_net::wire::{PROTOCOL, Status};

use crate::ping::line;

/// Returns the status of a server of `protocol` with 3 of 16 players.
fn status(protocol: u16) -> Status {
    Status {
        name: "Kodya".to_string(),
        players: 3,
        cap: 16,
        version: "0.9.0".to_string(),
        protocol,
    }
}

/// The line holds the name, the players, the version, the ping and the short key.
#[test]
fn a_status_prints_on_one_line() {
    let text = line(
        &status(PROTOCOL),
        Duration::from_millis(12),
        "7F3A-91C2-0B5E",
    );
    assert_eq!(
        text,
        "Kodya   3/16 players   v0.9.0   12 ms   key 7F3A-91C2-0B5E"
    );
}

/// A server of another protocol is flagged, so an outdated one is easy to spot.
#[test]
fn another_protocol_is_flagged() {
    let text = line(&status(PROTOCOL + 1), Duration::ZERO, "K");
    assert!(text.ends_with(&format!(
        "protocol {}, this program speaks {PROTOCOL}",
        PROTOCOL + 1
    )));
}
