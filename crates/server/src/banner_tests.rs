//! Tests of the banner: its lines, the LAN fallback and the password line.

use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

use crate::banner::{Banner, banner};

/// Returns a banner for a server on port 24650 at 192.168.1.20 with no password.
fn info() -> Banner<'static> {
    Banner {
        version: "0.10.0",
        name: "Kolva",
        seed: 42,
        file: Path::new("./data/world.toml"),
        port: 24650,
        lan: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))),
        key: "7F3A-91C2-0B5E",
        players: 16,
        password: false,
    }
}

/// Every line is there, with the labels in one column.
#[test]
fn the_banner_shows_how_to_join() {
    let expected = "\
Abstractions server v0.10.0
World     Kolva (seed 42)   ./data/world.toml
Listening UDP 24650 on all addresses (0.0.0.0)
Players   up to 16 at once
Join at   192.168.1.20:24650
Key       7F3A-91C2-0B5E   players see this the first time they join
Internet  forward UDP port 24650 to this machine, or point a domain at it
";
    assert_eq!(banner(&info()), expected);
}

/// Without a LAN address the banner says so in words, and a password shows as on only.
#[test]
fn the_banner_falls_back_and_hides_the_password() {
    let text = banner(&Banner {
        lan: None,
        password: true,
        ..info()
    });
    assert!(
        text.contains("\nJoin at   this machine's address, port 24650\n"),
        "{text}"
    );
    assert!(text.contains("\nPassword  on\n"), "{text}");
}
