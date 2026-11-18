//! A UDP relay between one player and a server, which can go silent to fake a dead server.

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A relay that can be cut.
pub struct Proxy {
    /// Address players connect to.
    pub addr: SocketAddr,
    /// Set to drop every packet from now on.
    cut: Arc<AtomicBool>,
}

impl Proxy {
    /// Starts relaying to `server`. The relay thread lives as long as the test process.
    pub fn start(server: SocketAddr) -> Proxy {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap();
        let cut = Arc::new(AtomicBool::new(false));
        let silent = cut.clone();
        std::thread::spawn(move || relay(&socket, server, &silent));
        Proxy { addr, cut }
    }

    /// Drops every packet from now on, both ways.
    pub fn cut(&self) {
        self.cut.store(true, Ordering::SeqCst);
    }
}

/// Sends what the server says to the last player heard, and what players say to the server.
fn relay(socket: &UdpSocket, server: SocketAddr, cut: &AtomicBool) {
    let mut packet = [0; 2048];
    let mut player = None;
    while let Ok((size, from)) = socket.recv_from(&mut packet) {
        let to = match from == server {
            true => player,
            false => player.replace(from).and(Some(server)).or(Some(server)),
        };
        if let Some(to) = to.filter(|_| !cut.load(Ordering::SeqCst)) {
            let _ = socket.send_to(&packet[..size], to);
        }
    }
}
