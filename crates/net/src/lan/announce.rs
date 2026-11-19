//! The host's side of the beacon: one broadcast per second from a small thread.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::Beacon;
use crate::wire::BEACON_PORT;

/// Time between two beacons.
const EVERY: Duration = Duration::from_secs(1);

/// A thread that broadcasts a beacon until it is stopped.
#[derive(Debug)]
pub struct Announcer {
    /// New beacons for the thread, or nothing to stop it.
    updates: Sender<Option<Beacon>>,
    /// The thread.
    thread: JoinHandle<()>,
}

impl Announcer {
    /// Starts broadcasting `beacon` to `255.255.255.255` on [`BEACON_PORT`].
    pub fn start(beacon: Beacon) -> io::Result<Self> {
        Self::start_to(beacon, SocketAddr::from((Ipv4Addr::BROADCAST, BEACON_PORT)))
    }

    /// Starts sending `beacon` to `target` once per second, such as a broadcast address of
    /// one network.
    pub fn start_to(beacon: Beacon, target: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))?;
        socket.set_broadcast(true)?;
        let (updates, inbox) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("abstractions-beacon".to_string())
            .spawn(move || announce(&socket, target, &beacon, &inbox))?;
        Ok(Self { updates, thread })
    }

    /// Replaces the beacon, such as when a player joins. It is sent at once.
    pub fn update(&self, beacon: Beacon) {
        let _ = self.updates.send(Some(beacon));
    }

    /// Stops broadcasting and waits for the thread to end.
    pub fn stop(self) {
        let _ = self.updates.send(None);
        let _ = self.thread.join();
    }
}

/// Sends the beacon every [`EVERY`] and at each update, until a stop. A failed send is
/// tried again next time: a network may come and go.
fn announce(
    socket: &UdpSocket,
    target: SocketAddr,
    first: &Beacon,
    inbox: &Receiver<Option<Beacon>>,
) {
    let mut packet = first.encode();
    loop {
        if let Some(packet) = &packet {
            let _ = socket.send_to(packet, target);
        }
        match inbox.recv_timeout(EVERY) {
            Ok(Some(beacon)) => packet = beacon.encode(),
            Ok(None) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}
