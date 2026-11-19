//! The status request: answered with the server's name and key, or a timeout when nobody is
//! there.

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use abstractions_net::io::{CloseReason, status};

use super::util::{scratch, start};

/// A server answers with its status and its fingerprint.
#[test]
fn a_server_tells_what_it_is() {
    let dir = scratch("status");
    let server = start(&dir, "127.0.0.1:0".parse::<SocketAddr>().unwrap());
    let address = server.local_addr().to_string();
    let (got, rtt, print) = status(&address, Duration::from_secs(5)).unwrap();
    assert_eq!(got.name, "test");
    assert_eq!(got.cap, 16);
    assert_eq!(print, server.fingerprint());
    assert!(rtt < Duration::from_secs(1));
    server.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

/// Nobody answering is a timeout after the time given.
#[test]
fn silence_is_a_timeout() {
    let quiet = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = quiet.local_addr().unwrap().to_string();
    let reason = status(&address, Duration::from_millis(500)).unwrap_err();
    assert_eq!(reason, CloseReason::Timeout);
}
