//! Tests of the beacon packet: the trip through bytes, the size cap and foreign packets.

use super::{BEACON_MAX, Beacon, MAGIC};

/// Returns a beacon with a name of `name_len` letters.
fn beacon(name_len: usize) -> Beacon {
    Beacon {
        protocol: 1,
        version: "0.9.0".to_string(),
        name: "w".repeat(name_len),
        players: 3,
        cap: 16,
        port: 24650,
        key: [9; 32],
    }
}

/// A beacon comes back from its bytes as it was, behind the magic.
#[test]
fn a_beacon_survives_its_bytes() {
    let packet = beacon(24).encode().unwrap();
    assert!(packet.starts_with(&MAGIC));
    assert!(packet.len() <= BEACON_MAX);
    assert_eq!(Beacon::decode(&packet), Some(beacon(24)));
}

/// A beacon too large to send is not encoded.
#[test]
fn a_huge_beacon_is_not_sent() {
    assert_eq!(beacon(400).encode(), None);
}

/// Other magics, cut packets, noise and oversized packets are not beacons.
#[test]
fn foreign_packets_are_ignored() {
    let packet = beacon(5).encode().unwrap();
    let mut other = packet.clone();
    other[3] = b'2';
    assert_eq!(Beacon::decode(&other), None);
    assert_eq!(Beacon::decode(&packet[..packet.len() - 1]), None);
    assert_eq!(Beacon::decode(b"ABS1"), None);
    assert_eq!(Beacon::decode(&[0xff; 40]), None);
    let mut long = packet;
    long.resize(BEACON_MAX + 1, 0);
    assert_eq!(Beacon::decode(&long), None);
}
