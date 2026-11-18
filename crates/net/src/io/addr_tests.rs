//! Tests of typed addresses: every accepted form, the default port and the errors.

use std::net::SocketAddr;

use super::addr::{AddrError, key, resolve};

/// Parses `text` as a socket address, for comparisons.
fn at(text: &str) -> SocketAddr {
    text.parse().unwrap()
}

/// Addresses with an IP resolve without the network, with or without a port.
#[test]
fn ip_addresses_resolve_with_the_default_port() {
    assert_eq!(resolve("1.2.3.4"), Ok(at("1.2.3.4:24650")));
    assert_eq!(resolve("  1.2.3.4:7000 "), Ok(at("1.2.3.4:7000")));
    assert_eq!(resolve("[::1]:7000"), Ok(at("[::1]:7000")));
    assert_eq!(resolve("[::1]"), Ok(at("[::1]:24650")));
    assert_eq!(resolve("::1"), Ok(at("[::1]:24650")));
}

/// `localhost` resolves, and to IPv4 when the system knows both.
#[test]
fn localhost_prefers_ipv4() {
    let found = resolve("localhost:9000").unwrap();
    assert_eq!(found.port(), 9000);
    assert!(found.ip().is_loopback());
    let all: Vec<_> = std::net::ToSocketAddrs::to_socket_addrs(&("localhost", 9000))
        .unwrap()
        .collect();
    assert_eq!(found.is_ipv4(), all.iter().any(SocketAddr::is_ipv4));
}

/// Bad text is refused with the right error, before any lookup.
#[test]
fn bad_text_is_refused() {
    assert_eq!(resolve("   "), Err(AddrError::Empty));
    assert_eq!(resolve("host:port"), Err(AddrError::BadPort));
    assert_eq!(resolve("host:0"), Err(AddrError::BadPort));
    assert_eq!(resolve("host:70000"), Err(AddrError::BadPort));
    assert_eq!(resolve(":24650"), Err(AddrError::Malformed));
    assert_eq!(resolve("two words"), Err(AddrError::Malformed));
    assert_eq!(resolve("a:b:1"), Err(AddrError::Malformed));
}

/// The key of an address is one written form whatever the player typed.
#[test]
fn keys_are_one_form() {
    assert_eq!(
        key(" Play.Example.org "),
        Ok("play.example.org:24650".to_string())
    );
    assert_eq!(
        key("play.example.org:24650"),
        Ok("play.example.org:24650".to_string())
    );
    assert_eq!(key("::1"), Ok("[::1]:24650".to_string()));
    assert_eq!(key("[::1]:24650"), Ok("[::1]:24650".to_string()));
    assert_eq!(key("10.0.0.2:5"), Ok("10.0.0.2:5".to_string()));
}
