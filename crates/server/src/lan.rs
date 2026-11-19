//! Finding this machine's address on its network, for the banner.

use std::net::{IpAddr, Ipv4Addr, UdpSocket};

/// A public address to route towards. Nothing is ever sent to it.
const PUBLIC: (Ipv4Addr, u16) = (Ipv4Addr::new(1, 1, 1, 1), 80);

/// Returns the address the system would send from to reach the internet, which is the one
/// other machines on the network reach this one at. A UDP `connect` only picks a route; no
/// packet leaves. Returns nothing without a route.
pub fn lan_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect(PUBLIC).ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_unspecified() && !ip.is_loopback()).then_some(ip)
}
