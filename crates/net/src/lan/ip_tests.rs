//! Tests of the choice of the address of this machine, with the routes made up.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ip::{PROBES, first_reachable, reachable};

/// Returns the address `a.b.c.d` as a route gives it.
fn at(a: u8, b: u8, c: u8, d: u8) -> Option<IpAddr> {
    Some(IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
}

/// The loopback and the unspecified address, in both families, are not reachable from another
/// machine, and an address of a network is.
#[test]
fn only_a_network_address_is_reachable() {
    for ip in [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::UNSPECIFIED),
    ] {
        assert_eq!(reachable(ip), None);
    }
    assert_eq!(reachable(at(192, 168, 1, 20).unwrap()), at(192, 168, 1, 20));
}

/// The first probe that gives a reachable address wins, and the ones after it are not asked.
#[test]
fn the_first_reachable_route_wins() {
    let asked = std::cell::RefCell::new(Vec::new());
    let found = first_reachable(|probe| {
        asked.borrow_mut().push(probe);
        at(10, 1, 2, 3)
    });
    assert_eq!(found, at(10, 1, 2, 3));
    assert_eq!(*asked.borrow(), [PROBES[0]]);
}

/// A probe with no route, or a route out of the loopback, is skipped for the next one.
#[test]
fn a_probe_without_a_route_is_skipped() {
    let found = first_reachable(|probe| match probe {
        probe if probe == PROBES[0] => None,
        probe if probe == PROBES[1] => Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        _ => at(172, 16, 4, 9),
    });
    assert_eq!(found, at(172, 16, 4, 9));
}

/// A machine with no route at all has no address: the caller decides what to say.
#[test]
fn no_route_means_no_address() {
    assert_eq!(first_reachable(|_| None), None);
}
