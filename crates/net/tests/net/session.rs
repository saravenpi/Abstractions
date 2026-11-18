//! Two players on one server: welcome, messages, datagrams, a refusal and a clean stop.

use std::net::SocketAddr;
use std::time::Duration;

use abstractions_net::io::{Client, ClientEvent, CloseReason};
use abstractions_net::wire::{ClientMsg, Reason, ServerMsg};

use super::util::{closed, connected, opts, scratch, start, until};

/// Loopback with a port the system picks.
const ANY: &str = "127.0.0.1:0";

/// Waits for the welcome, pings, and bounces a datagram off the server.
fn play(client: &mut Client, id: u32) {
    let ten = Duration::from_secs(10);
    until(client, ten, |event| {
        matches!(event, ClientEvent::Message(ServerMsg::Welcome(_))).then_some(())
    });
    client.send(ClientMsg::Ping { id, sent: 1.5 });
    until(client, ten, |event| match event {
        ClientEvent::Message(ServerMsg::Pong { id: got, sent, .. }) if got == id => Some(sent),
        _ => None,
    });
    assert!(client.max_datagram() > 1000);
    assert!(client.rtt().is_some());
    let bytes = id.to_le_bytes();
    for _ in 0..50 {
        client.datagram(&bytes);
        let back = until(client, ten, |event| match event {
            ClientEvent::Datagram(got) => Some(got),
            _ => None,
        });
        if back == bytes {
            return;
        }
    }
    panic!("the datagram never came back");
}

/// Two players connect at once, each gets its welcome, its pongs and its own datagrams.
#[test]
fn two_players_play_together() {
    let dir = scratch("two");
    let server = start(&dir, ANY.parse::<SocketAddr>().unwrap());
    let address = server.local_addr().to_string();
    let mut one = Client::connect(&address, opts(&dir.join("one.toml"), "one"));
    let mut two = Client::connect(&address, opts(&dir.join("two.toml"), "two"));
    assert!(connected(&mut one));
    assert!(connected(&mut two));
    play(&mut one, 1);
    play(&mut two, 2);
    one.close();
    two.close();
    let host = server.stop();
    assert_eq!(host.seen, 2);
    assert!(host.open.is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}

/// A refused player reads why, and its connection ends with that reason.
#[test]
fn a_refused_player_learns_why() {
    let dir = scratch("refused");
    let server = start(&dir, ANY.parse::<SocketAddr>().unwrap());
    let address = server.local_addr().to_string();
    let mut client = Client::connect(&address, opts(&dir.join("known.toml"), "full"));
    connected(&mut client);
    let five = Duration::from_secs(5);
    until(&mut client, five, |event| {
        (event == ClientEvent::Message(ServerMsg::Refused(Reason::Full))).then_some(())
    });
    assert_eq!(
        closed(&mut client, five),
        CloseReason::Refused(Reason::Full)
    );
    server.stop();
    std::fs::remove_dir_all(dir).unwrap();
}

/// Players in the world when the server stops are told it is closing. A player still in its
/// handshake would only see the connection end, so the test waits for the welcome.
#[test]
fn a_stopping_server_says_it_closes() {
    let dir = scratch("stopping");
    let server = start(&dir, ANY.parse::<SocketAddr>().unwrap());
    let address = server.local_addr().to_string();
    let mut client = Client::connect(&address, opts(&dir.join("known.toml"), "stay"));
    connected(&mut client);
    until(&mut client, Duration::from_secs(5), |event| {
        matches!(event, ClientEvent::Message(ServerMsg::Welcome(_))).then_some(())
    });
    let host = server.stop();
    assert!(host.open.is_empty());
    let reason = closed(&mut client, Duration::from_secs(5));
    assert_eq!(reason, CloseReason::Refused(Reason::Closing));
    std::fs::remove_dir_all(dir).unwrap();
}

/// A name that resolves nowhere ends at once with an address error.
#[test]
fn a_bad_address_ends_at_once() {
    let dir = scratch("bad");
    let mut client = Client::connect("host:port", opts(&dir.join("known.toml"), "lost"));
    let reason = closed(&mut client, Duration::from_secs(5));
    assert!(matches!(reason, CloseReason::Address(_)), "{reason:?}");
}
