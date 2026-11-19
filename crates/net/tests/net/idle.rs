//! A server that stops answering: the player notices within the idle timeout.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use abstractions_net::io::{Client, CloseReason};

use super::proxy::Proxy;
use super::util::{closed, connected, opts, scratch, start};

/// The player reports a timeout 15 to 21 seconds after the server went silent: the idle timer
/// starts again at the first keep-alive sent after the last packet heard, up to 5 seconds later,
/// and the session lingers 1 second before it reports.
#[test]
fn a_silent_server_times_out() {
    let dir = scratch("idle");
    let server = start(
        &dir.join("data"),
        "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
    );
    let proxy = Proxy::start(server.local_addr());
    let mut client = Client::connect(&proxy.addr.to_string(), opts(&dir.join("known.toml"), "a"));
    connected(&mut client);
    proxy.cut();
    let cut = Instant::now();
    let reason = closed(&mut client, Duration::from_secs(25));
    let waited = cut.elapsed();
    assert_eq!(reason, CloseReason::Timeout);
    assert!(waited > Duration::from_secs(10), "{waited:?}");
    server.stop();
    std::fs::remove_dir_all(dir).unwrap();
}
