//! Where the events of a link come from: a real connection, or a script in tests.

use std::thread;

use abstractions_net::io::{Client, ClientEvent};
use abstractions_net::wire::ClientMsg;

#[cfg(test)]
use std::collections::VecDeque;

/// What a scripted connection hands out and what the game sent through it, in tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct Wire {
    /// Events still to hand out, in order.
    pub events: VecDeque<ClientEvent>,
    /// Messages the game sent, oldest first.
    pub sent: Vec<ClientMsg>,
    /// Datagrams the game sent, oldest first.
    pub datagrams: Vec<Vec<u8>>,
}

/// A scripted connection, shared with the test that drives it.
#[cfg(test)]
pub type Script = std::sync::Arc<std::sync::Mutex<Wire>>;

/// The other end of a link.
#[derive(Debug)]
pub enum Source {
    /// A connection to a server, on its own thread.
    Live(Client),
    /// A script, in tests.
    #[cfg(test)]
    Script(Script),
}

impl Source {
    /// Returns the next event, or nothing yet. Never blocks.
    pub fn poll(&mut self) -> Option<ClientEvent> {
        match self {
            Self::Live(client) => client.poll(),
            #[cfg(test)]
            Self::Script(script) => script.lock().ok()?.events.pop_front(),
        }
    }

    /// Queues a message for the server.
    pub fn send(&mut self, msg: ClientMsg) {
        match self {
            Self::Live(client) => client.send(msg),
            #[cfg(test)]
            Self::Script(script) => {
                if let Ok(mut wire) = script.lock() {
                    wire.sent.push(msg);
                }
            }
        }
    }

    /// Sends a datagram, dropped when it does not fit.
    pub fn datagram(&mut self, bytes: &[u8]) {
        match self {
            Self::Live(client) => client.datagram(bytes),
            #[cfg(test)]
            Self::Script(script) => {
                if let Ok(mut wire) = script.lock() {
                    wire.datagrams.push(bytes.to_vec());
                }
            }
        }
    }

    /// Returns the most bytes a datagram may carry now, 0 before the connection is up.
    pub fn max_datagram(&self) -> usize {
        match self {
            Self::Live(client) => client.max_datagram(),
            #[cfg(test)]
            Self::Script(_) => abstractions_net::host::DATAGRAM_BUDGET,
        }
    }

    /// Says goodbye and ends the connection. With `wait` it returns once the connection
    /// thread is over, else a small thread waits for it so no frame does.
    pub fn close(self, wait: bool) {
        match self {
            Self::Live(client) if wait => client.close(),
            Self::Live(client) => {
                let _ = thread::Builder::new()
                    .name("abstractions-bye".to_string())
                    .spawn(move || client.close());
            }
            #[cfg(test)]
            Self::Script(_) => {}
        }
    }
}
