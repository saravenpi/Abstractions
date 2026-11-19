//! Trust on first use across real connections: remember, accept again, refuse a new key.

use std::net::SocketAddr;
use std::time::Duration;

use abstractions_net::io::{Client, CloseReason};
use abstractions_net::trust::{KnownServers, TrustError};

use super::util::{closed, connected, opts, scratch, start};

/// The first connection remembers the key, the next one is accepted, and a server with a new
/// identity on the same address is refused.
#[test]
fn a_changed_key_is_refused() {
    let dir = scratch("changed");
    let known = dir.join("known_servers.toml");
    let first = start(
        &dir.join("first"),
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
    );
    let addr = first.local_addr();
    let mut client = Client::connect(&addr.to_string(), opts(&known, "a"));
    assert!(connected(&mut client));
    client.close();
    let stored = KnownServers::load(&known).unwrap().get(&addr.to_string());
    assert_eq!(stored, Some(first.fingerprint()));
    let mut again = Client::connect(&addr.to_string(), opts(&known, "a"));
    assert!(!connected(&mut again));
    again.close();
    let expected = first.fingerprint();
    first.stop();
    let second = start(&dir.join("second"), addr);
    let mut refused = Client::connect(&addr.to_string(), opts(&known, "a"));
    let reason = closed(&mut refused, Duration::from_secs(10));
    let got = second.fingerprint();
    assert_eq!(
        reason,
        CloseReason::Trust(TrustError::Changed { expected, got })
    );
    second.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

/// A server restarted on the same folder keeps its key, and players accept it.
#[test]
fn a_restart_keeps_the_key() {
    let dir = scratch("restart");
    let known = dir.join("known_servers.toml");
    let server = start(
        &dir.join("data"),
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
    );
    let (addr, print) = (server.local_addr(), server.fingerprint());
    let mut client = Client::connect(&addr.to_string(), opts(&known, "a"));
    assert!(connected(&mut client));
    client.close();
    server.stop();
    let again = start(&dir.join("data"), addr);
    assert_eq!(again.fingerprint(), print);
    let mut client = Client::connect(&addr.to_string(), opts(&known, "a"));
    assert!(!connected(&mut client));
    client.close();
    again.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

/// A key announced by a beacon must be the one the server shows.
#[test]
fn an_announced_key_must_match() {
    let dir = scratch("announced");
    let server = start(
        &dir.join("data"),
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
    );
    let mut wrong = opts(&dir.join("known.toml"), "a");
    wrong.expected = Some([0; 32]);
    let mut client = Client::connect(&server.local_addr().to_string(), wrong);
    let reason = closed(&mut client, Duration::from_secs(10));
    let got = server.fingerprint();
    let expected = [0; 32];
    assert_eq!(
        reason,
        CloseReason::Trust(TrustError::Changed { expected, got })
    );
    server.stop();
    std::fs::remove_dir_all(dir).unwrap();
}
