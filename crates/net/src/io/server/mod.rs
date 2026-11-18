//! The server side: one thread that takes connections and drives a [`Host`].
//!
//! The host and every connection task live on that thread. quinn's own endpoint and
//! connection drivers run on smol's global executor, as `quinn::SmolRuntime` spawns them there.

mod conn;
mod run;
mod tick;

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use smol::channel::{Sender, bounded};

use super::config;
use crate::host::Host;
use crate::trust::Identity;
use run::Event;

/// Events waiting for the host at most: enough for a burst of 16 players.
const EVENTS: usize = 1024;

/// Longest wait for the port to be free after a stop.
const RELEASE: Duration = Duration::from_secs(2);

/// Where a server listens and keeps its certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeConfig {
    /// Address and UDP port to listen on, such as `0.0.0.0:24650`.
    pub bind: SocketAddr,
    /// Folder of `cert.der` and `key.der`, made on the first start.
    pub dir: PathBuf,
}

/// A running server. Dropping it leaves the server running; [`Self::stop`] ends it.
#[derive(Debug)]
pub struct ServerHandle<H> {
    /// Address the server listens on, with the port the system gave.
    addr: SocketAddr,
    /// Fingerprint of the server's certificate.
    fingerprint: [u8; 32],
    /// Way to tell the server's thread to stop.
    events: Sender<Event>,
    /// The server's thread, which gives the host back when it ends.
    thread: JoinHandle<H>,
}

/// Starts a server on its own thread and returns at once.
///
/// The certificate is loaded from `config.dir`, or made there on the first start. The thread
/// ticks `host` at [`crate::host::TICK_HZ`] while [`Host::busy`] says so and sleeps otherwise.
pub fn serve<H: Host + Send + 'static>(
    config: ServeConfig,
    host: H,
) -> io::Result<ServerHandle<H>> {
    let identity = Identity::load_or_create(&config.dir)?;
    let endpoint = config::endpoint(config.bind, Some(config::server(&identity)?))?;
    let addr = endpoint.local_addr()?;
    let (events, inbox) = bounded(EVENTS);
    let sender = events.clone();
    let thread = thread::Builder::new()
        .name("abstractions-server".to_string())
        .spawn(move || run::run(endpoint, host, sender, inbox))?;
    Ok(ServerHandle {
        addr,
        fingerprint: identity.fingerprint(),
        events,
        thread,
    })
}

impl<H> ServerHandle<H> {
    /// Returns the address the server listens on.
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the fingerprint of the server's certificate.
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    /// Closes every connection with [`crate::wire::Reason::Closing`], ends the thread and
    /// gives the host back, so the caller can save the world.
    ///
    /// It returns once the port is free again, so a new server can listen on it at once.
    pub fn stop(self) -> H {
        let _ = self.events.send_blocking(Event::Stop);
        let host = self
            .thread
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        released(self.addr);
        host
    }
}

/// Waits, for at most [`RELEASE`], until `addr` can be bound again. quinn drops the socket
/// from its own task a moment after the last handle goes.
fn released(addr: SocketAddr) {
    let end = Instant::now() + RELEASE;
    while UdpSocket::bind(addr).is_err() && Instant::now() < end {
        thread::sleep(Duration::from_millis(5));
    }
}
