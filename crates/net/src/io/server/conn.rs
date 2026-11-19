//! One task per connection: the handshake, the stream, the datagrams, and the news of each to
//! the loop.

use std::time::Duration;

use quinn::{Connection, Endpoint, Incoming, RecvStream, SendStream};
use smol::channel::{Sender, bounded};
use smol::{Executor, Timer, future};

use super::run::{Event, Link};
use crate::host::ConnId;
use crate::io::close::{BROKEN, BYE};
use crate::io::link::{self, End};
use crate::wire::{CLIENT_MESSAGE_MAX, ClientMsg};

/// Messages waiting for a connection's stream at most: one from each of
/// [`crate::wire::MAX_PLAYERS_LIMIT`] other players at once. A player that lets more pile up
/// is closed as too slow.
const QUEUE: usize = 1024;

/// How long a new connection may take to open its stream.
const OPEN_TIMEOUT: Duration = Duration::from_secs(10);

/// Takes connections until the endpoint closes. An address that has not proved it can receive
/// is sent a retry first, so a spoofed address cannot make the server work.
pub(super) async fn accept(
    executor: &Executor<'static>,
    endpoint: Endpoint,
    events: Sender<Event>,
) {
    let mut next = 0;
    while let Some(incoming) = endpoint.accept().await {
        if !incoming.remote_address_validated() {
            let _ = incoming.retry();
            continue;
        }
        next += 1;
        executor
            .spawn(connection(ConnId(next), incoming, events.clone()))
            .detach();
    }
}

/// Runs one connection from its handshake to its end, then tells the loop it is gone.
async fn connection(id: ConnId, incoming: Incoming, events: Sender<Event>) {
    let Ok(connecting) = incoming.accept() else {
        return;
    };
    let Ok(conn) = connecting.await else {
        return;
    };
    let Some((send, recv)) = open(conn.clone()).await else {
        conn.close(BROKEN, b"no stream");
        return;
    };
    let (out, queue) = bounded(QUEUE);
    let link = Link {
        conn: conn.clone(),
        out,
    };
    if events.send(Event::Connected(id, link)).await.is_err() {
        return;
    }
    let reader = read(id, recv, events.clone());
    let datagrams = datagrams(id, conn.clone(), events.clone());
    let writer = link::write(conn.clone(), send, queue);
    match future::or(writer, future::or(reader, datagrams)).await {
        End::Broken => conn.close(BROKEN, b"malformed message"),
        End::Over | End::Stopped => conn.close(BYE, b"bye"),
    }
    let _ = events.send(Event::Closed(id)).await;
}

/// Waits for the stream the player opens first, for at most [`OPEN_TIMEOUT`].
async fn open(conn: Connection) -> Option<(SendStream, RecvStream)> {
    let stream = async { conn.accept_bi().await.ok() };
    let late = async {
        Timer::after(OPEN_TIMEOUT).await;
        None
    };
    future::or(stream, late).await
}

/// Hands each message of the stream to the loop, in order.
async fn read(id: ConnId, recv: RecvStream, events: Sender<Event>) -> End {
    let each = async move |msg: ClientMsg| events.send(Event::Message(id, msg)).await.is_ok();
    link::read(recv, CLIENT_MESSAGE_MAX, each).await
}

/// Hands each datagram to the loop, dropped when the loop is behind: a newer one follows.
async fn datagrams(id: ConnId, conn: Connection, events: Sender<Event>) -> End {
    while let Ok(bytes) = conn.read_datagram().await {
        let _ = events.try_send(Event::Datagram(id, bytes));
    }
    End::Over
}
