//! The player side: a connection on its own thread, read and written from the game loop
//! without ever blocking it.

mod dial;
mod session;
mod status;

pub use status::status;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use quinn::Connection;
use smol::channel::{Receiver, Sender, bounded};

use super::addr::AddrError;
use super::close::LAGGING;
use super::link::Out;
use crate::trust::TrustError;
use crate::wire::{ClientMsg, Hello, Reason, ServerMsg, encode};
use session::Session;

/// Events waiting for the game at most, and messages waiting for the server at most.
const QUEUE: usize = 1024;

/// What a player needs to join a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectOpts {
    /// The first message, sent as soon as the stream is open.
    pub hello: Hello,
    /// File of the known servers, such as `known_servers.toml`.
    pub known: PathBuf,
    /// Fingerprint the server must show, such as the one its LAN beacon announced.
    pub expected: Option<[u8; 32]>,
}

/// Why a connection ended or never started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseReason {
    /// The server refused the player or sent it away.
    Refused(Reason),
    /// The player does not trust the server.
    Trust(TrustError),
    /// The address names no server.
    Address(AddrError),
    /// The server stopped answering.
    Timeout,
    /// The server never answered.
    Unreachable,
    /// The connection broke for another reason, or the player closed it.
    Lost,
}

/// What happened on a connection, in order.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// The server is trusted and the hello is on its way.
    Connected {
        /// Fingerprint of the server's certificate.
        fingerprint: [u8; 32],
        /// True when the server was new and is now remembered.
        first_time: bool,
    },
    /// A message from the server.
    Message(ServerMsg),
    /// A datagram from the server.
    Datagram(Vec<u8>),
    /// The connection is over. Nothing follows.
    Closed(CloseReason),
}

/// A connection to a server, running on its own thread.
#[derive(Debug)]
pub struct Client {
    /// What the connection thread reports.
    events: Receiver<ClientEvent>,
    /// Queue of the stream writer.
    out: Sender<Out>,
    /// The connection, once it is up, for datagrams and measures.
    link: Arc<OnceLock<Connection>>,
    /// The connection thread.
    thread: Option<JoinHandle<()>>,
}

impl Client {
    /// Starts connecting to `address` and returns at once. The thread resolves the address,
    /// connects, checks the server's key and sends the hello of `opts`. Follow it with
    /// [`Self::poll`].
    pub fn connect(address: &str, opts: ConnectOpts) -> Client {
        let (report, events) = bounded(QUEUE);
        let (out, queue) = bounded(QUEUE);
        let link = Arc::new(OnceLock::new());
        let session = Session::new(address, opts, report.clone(), queue, link.clone());
        let thread = thread::Builder::new()
            .name("abstractions-client".to_string())
            .spawn(move || session.run());
        if thread.is_err() {
            let _ = report.try_send(ClientEvent::Closed(CloseReason::Lost));
        }
        Client {
            events,
            out,
            link,
            thread: thread.ok(),
        }
    }

    /// Returns the next event, or nothing when there is none yet. Never blocks.
    pub fn poll(&mut self) -> Option<ClientEvent> {
        self.events.try_recv().ok()
    }

    /// Queues a message for the server. When too many wait, the connection cannot keep up and
    /// is closed.
    pub fn send(&self, msg: ClientMsg) {
        let Ok(frame) = encode(&msg) else {
            return;
        };
        if self.out.try_send(Out::Frame(frame)).is_err()
            && let Some(conn) = self.link.get()
        {
            conn.close(LAGGING, b"too slow");
        }
    }

    /// Sends a datagram, or drops it when the connection is not up or it is longer than
    /// [`Self::max_datagram`].
    pub fn datagram(&self, bytes: &[u8]) {
        if let Some(conn) = self.link.get()
            && bytes.len() <= self.max_datagram()
        {
            let _ = conn.send_datagram(bytes::Bytes::copy_from_slice(bytes));
        }
    }

    /// Returns the most bytes a datagram may carry now, or 0 before the connection is up.
    pub fn max_datagram(&self) -> usize {
        self.link
            .get()
            .and_then(Connection::max_datagram_size)
            .unwrap_or(0)
    }

    /// Returns the round trip time measured by QUIC, once the connection is up.
    pub fn rtt(&self) -> Option<Duration> {
        self.link.get().map(Connection::rtt)
    }

    /// Says goodbye to the server and waits for the connection thread to end.
    pub fn close(self) {
        self.out.close();
        self.events.close();
        if let Some(thread) = self.thread {
            let _ = thread.join();
        }
    }
}
