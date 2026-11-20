//! This machine's address on its local network.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

/// The addresses to route towards, one in each private range. Nothing is ever sent to them: a
/// machine that is on the network of one has a route to it, and a machine with a default route
/// answers for all three.
pub(super) const PROBES: [Ipv4Addr; 3] = [
    Ipv4Addr::new(192, 168, 0, 1),
    Ipv4Addr::new(10, 0, 0, 1),
    Ipv4Addr::new(172, 16, 0, 1),
];

/// Port of the probes. It is never opened: a UDP `connect` only picks a route.
const PROBE_PORT: u16 = 9;

/// Returns the address the system would send from to reach `target`. A UDP `connect` sends
/// nothing, so no packet leaves. Returns nothing without a route.
fn route(target: Ipv4Addr) -> Option<IpAddr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket
        .connect(SocketAddr::from((target, PROBE_PORT)))
        .ok()?;
    Some(socket.local_addr().ok()?.ip())
}

/// Returns `ip` when other machines can reach this one at it: not the unspecified address and
/// not the loopback address.
pub(super) fn reachable(ip: IpAddr) -> Option<IpAddr> {
    (!ip.is_unspecified() && !ip.is_loopback()).then_some(ip)
}

/// Returns the first reachable address that `route` gives for the [`PROBES`].
pub(super) fn first_reachable(route: impl Fn(Ipv4Addr) -> Option<IpAddr>) -> Option<IpAddr> {
    PROBES
        .into_iter()
        .find_map(|probe| route(probe).and_then(reachable))
}

/// Returns the address of this machine on its local network, or nothing when it has none. The
/// caller says what to use then.
pub fn lan_ip() -> Option<IpAddr> {
    first_reachable(route)
}
