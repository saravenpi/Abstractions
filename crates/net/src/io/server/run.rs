//! The server's loop: the one place where the host lives, fed by every connection.

use std::collections::HashMap;
use std::time::Instant;

use bytes::Bytes;
use quinn::{Connection, Endpoint};
use smol::channel::{Receiver, Sender};
use smol::{Executor, future};

use super::conn;
use super::tick::{Ticker, Wake};
use crate::host::{ConnId, Host, Op, Outbox};
use crate::io::close::{LAGGING, code, text};
use crate::io::link::{Out, linger};
use crate::wire::{Reason, ServerMsg, encode};

/// What a connection tells the loop.
#[derive(Debug)]
pub(crate) enum Event {
    /// A player connected and opened its stream.
    Connected(ConnId, Link),
    /// A whole message arrived on the stream.
    Message(ConnId, crate::wire::ClientMsg),
    /// A datagram arrived.
    Datagram(ConnId, Bytes),
    /// The connection is gone.
    Closed(ConnId),
    /// The owner asked the server to stop.
    Stop,
}

/// What the loop holds of a connection.
#[derive(Debug)]
pub(crate) struct Link {
    /// The connection, for datagrams and closes.
    pub(crate) conn: Connection,
    /// Queue of the connection's stream writer.
    pub(crate) out: Sender<Out>,
}

/// The host and everything around it.
struct Server<H> {
    /// The server's logic.
    host: H,
    /// Every connection the host knows, by id.
    links: HashMap<ConnId, Link>,
    /// What the host asked for, reused from call to call.
    out: Outbox,
    /// When the server started: the host's time counts from here.
    start: Instant,
}

/// Runs the server until it is told to stop, then gives the host back. Connections run as
/// tasks of one executor on this thread, which drops them when the loop ends.
pub(super) fn run<H: Host>(
    endpoint: Endpoint,
    host: H,
    events: Sender<Event>,
    inbox: Receiver<Event>,
) -> H {
    let executor = Executor::new();
    let server = Server {
        host,
        links: HashMap::new(),
        out: Outbox::default(),
        start: Instant::now(),
    };
    let accept = conn::accept(&executor, endpoint.clone(), events);
    let accept = async {
        accept.await;
        future::pending().await
    };
    smol::block_on(executor.run(future::or(server.serve(endpoint, inbox), accept)))
}

impl<H: Host> Server<H> {
    /// Hands every event and tick to the host and sends what it asks, until a stop.
    async fn serve(mut self, endpoint: Endpoint, inbox: Receiver<Event>) -> H {
        let mut ticker = Ticker::new();
        loop {
            let wake = ticker.wait(&inbox, self.host.busy()).await;
            let now = self.start.elapsed().as_secs_f64();
            match wake {
                Wake::Tick => self.host.tick(now, &mut self.out),
                Wake::Event(Event::Stop) => break,
                Wake::Event(event) => self.handle(event, now),
            }
            self.flush();
        }
        self.shutdown(endpoint).await
    }

    /// Hands one event to the host. Events of a connection it no longer knows are dropped.
    fn handle(&mut self, event: Event, now: f64) {
        let (host, out) = (&mut self.host, &mut self.out);
        match event {
            Event::Connected(id, link) => {
                let addr = link.conn.remote_address();
                self.links.insert(id, link);
                host.connected(id, addr, now, out);
            }
            Event::Message(id, msg) if self.links.contains_key(&id) => {
                host.message(id, msg, now, out)
            }
            Event::Datagram(id, bytes) if self.links.contains_key(&id) => {
                host.datagram(id, &bytes, now, out)
            }
            Event::Closed(id) if self.links.remove(&id).is_some() => host.closed(id, now, out),
            _ => {}
        }
    }

    /// Carries out what the host asked, then empties the outbox. Nothing here waits.
    fn flush(&mut self) {
        for op in self.out.ops() {
            match op {
                Op::Send(id, msg) => send(self.links.get(id), msg),
                Op::Datagram(id, range) => {
                    if let Some(link) = self.links.get(id) {
                        let bytes = Bytes::copy_from_slice(self.out.bytes(range.clone()));
                        let _ = link.conn.send_datagram(bytes);
                    }
                }
                Op::Close(id, reason) => close(self.links.get(id), *reason),
            }
        }
        self.out.clear();
    }

    /// Closes every connection as the server closes and gives the host back.
    async fn shutdown(mut self, endpoint: Endpoint) -> H {
        let now = self.start.elapsed().as_secs_f64();
        for (id, link) in self.links.drain() {
            link.conn
                .close(code(Reason::Closing), text(Reason::Closing));
            self.host.closed(id, now, &mut self.out);
        }
        self.out.clear();
        endpoint.close(code(Reason::Closing), text(Reason::Closing));
        future::or(endpoint.wait_idle(), linger()).await;
        self.host
    }
}

/// Queues a message on a connection's stream. A connection whose queue is full cannot keep
/// up: it is closed rather than buffered forever.
fn send(link: Option<&Link>, msg: &ServerMsg) {
    let (Some(link), Ok(frame)) = (link, encode(msg)) else {
        return;
    };
    if link.out.try_send(Out::Frame(frame)).is_err() {
        link.conn.close(LAGGING, b"too slow");
    }
}

/// Closes a connection for `reason` once its stream is written out, or at once when its
/// queue is full.
fn close(link: Option<&Link>, reason: Reason) {
    let Some(link) = link else {
        return;
    };
    let order = Out::Close(code(reason), text(reason));
    if link.out.try_send(order).is_err() {
        link.conn.close(code(reason), text(reason));
    }
}
