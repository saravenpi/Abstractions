//! The LAN beacon end to end: a listener hears an announcer within three seconds.
//!
//! The beacon goes to `127.0.0.1` rather than `255.255.255.255`: on a machine whose only
//! broadcast route leaves by a wired network, a broadcast never comes back to itself.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

use abstractions_net::lan::{Announcer, Beacon, Listener};
use abstractions_net::wire::BEACON_PORT;

/// Returns a beacon with a name only this test uses.
fn beacon(players: u8) -> Beacon {
    Beacon {
        protocol: 1,
        version: "0.9.0".to_string(),
        name: format!("beacon-test-{}", std::process::id()),
        players,
        cap: 16,
        port: 24650,
        key: [3; 32],
    }
}

/// Polls `listener` until it lists `want`, for at most three seconds.
fn hears(listener: &mut Listener, want: &Beacon) -> bool {
    let end = Instant::now() + Duration::from_secs(3);
    while Instant::now() < end {
        if listener.poll().iter().any(|found| found.beacon == *want) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// A beacon is heard, and an update replaces it.
#[test]
fn a_listener_hears_an_announcer() {
    let mut listener = Listener::start().unwrap();
    let target = SocketAddr::from((Ipv4Addr::LOCALHOST, BEACON_PORT));
    let announcer = Announcer::start_to(beacon(1), target).unwrap();
    assert!(hears(&mut listener, &beacon(1)));
    announcer.update(beacon(2));
    assert!(hears(&mut listener, &beacon(2)));
    let found = listener.poll();
    let ours: Vec<_> = found
        .iter()
        .filter(|f| f.beacon.name == beacon(2).name)
        .collect();
    assert_eq!(ours.len(), 1);
    assert_eq!(ours[0].addr.port(), 24650);
    announcer.stop();
}
