//! The finder's side of the beacon: a socket on the beacon port, read when the game asks.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use socket2::{Domain, Protocol, Socket, Type};

use super::{BEACON_MAX, Beacon};
use crate::wire::BEACON_PORT;

/// How long a host stays listed after its last beacon.
const EXPIRY: Duration = Duration::from_secs(4);

/// A host heard on the local network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Where to connect: the sender's address and the beacon's port.
    pub addr: SocketAddr,
    /// What the host said last.
    pub beacon: Beacon,
    /// When it was heard last.
    pub seen: Instant,
}

/// Listens for beacons without a thread: [`Self::poll`] reads what arrived.
#[derive(Debug)]
pub struct Listener {
    /// Non-blocking socket on the beacon port, shared with other listeners on this machine.
    socket: UdpSocket,
    /// Hosts heard recently.
    found: Vec<Found>,
}

impl Listener {
    /// Listens on `0.0.0.0` at [`BEACON_PORT`]. Several games on one machine can listen at
    /// once.
    pub fn start() -> io::Result<Self> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        #[cfg(unix)]
        socket.set_reuse_port(true)?;
        socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, BEACON_PORT)).into())?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket: socket.into(),
            found: Vec::new(),
        })
    }

    /// Reads every beacon that arrived, forgets hosts silent for 4 seconds, and returns the
    /// hosts heard. Never blocks. Packets that are not beacons are ignored.
    pub fn poll(&mut self) -> Vec<Found> {
        let mut packet = [0; BEACON_MAX + 1];
        let now = Instant::now();
        while let Ok((size, from)) = self.socket.recv_from(&mut packet) {
            if let Some(beacon) = Beacon::decode(&packet[..size]) {
                self.heard(from, beacon, now);
            }
        }
        self.found.retain(|found| now - found.seen < EXPIRY);
        self.found.clone()
    }

    /// Records a beacon from `from`, replacing what the same host said before.
    fn heard(&mut self, from: SocketAddr, beacon: Beacon, now: Instant) {
        let addr = SocketAddr::new(from.ip(), beacon.port);
        self.found.retain(|found| found.addr != addr);
        self.found.push(Found {
            addr,
            beacon,
            seen: now,
        });
    }
}
