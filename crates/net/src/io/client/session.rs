//! The connection thread of a player: join, then carry messages and datagrams both ways.

use std::io;
use std::sync::{Arc, OnceLock};

use quinn::{Connection, Endpoint};
use smol::channel::{Receiver, Sender};
use smol::future;

use super::dial::{dial, ended};
use super::{ClientEvent, CloseReason, ConnectOpts};
use crate::io::addr;
use crate::io::close::{BROKEN, BYE};
use crate::io::link::{self, End, Out, linger};
use crate::trust::{KnownServers, Seen, TrustError, Verifier};
use crate::wire::{ClientMsg, MESSAGE_MAX, ServerMsg, encode};

/// Everything the connection thread works with.
#[derive(Debug)]
pub(super) struct Session {
    /// The address as the player typed it.
    address: String,
    /// The hello and the trust settings.
    opts: ConnectOpts,
    /// Where events go, for the game to poll.
    report: Sender<ClientEvent>,
    /// What the game wants written on the stream.
    queue: Receiver<Out>,
    /// The connection, shared with the game once it is up.
    link: Arc<OnceLock<Connection>>,
}

impl Session {
    /// Gathers what the connection thread needs.
    pub(super) fn new(
        address: &str,
        opts: ConnectOpts,
        report: Sender<ClientEvent>,
        queue: Receiver<Out>,
        link: Arc<OnceLock<Connection>>,
    ) -> Self {
        Self {
            address: address.to_string(),
            opts,
            report,
            queue,
            link,
        }
    }

    /// Runs the connection to its end, then reports why it ended.
    pub(super) fn run(self) {
        let reason = smol::block_on(self.go());
        let _ = self.report.send_blocking(ClientEvent::Closed(reason));
    }

    /// Joins, lives, and lets the last packets leave before the endpoint goes.
    async fn go(&self) -> CloseReason {
        let (endpoint, conn, seen) = match self.join().await {
            Ok(joined) => joined,
            Err(reason) => return reason,
        };
        let reason = self.live(&conn, seen).await;
        future::or(endpoint.wait_idle(), linger()).await;
        reason
    }

    /// Connects and checks the server's key, remembering a server seen the first time.
    async fn join(&self) -> Result<(Endpoint, Connection, Seen), CloseReason> {
        let key = addr::key(&self.address).map_err(CloseReason::Address)?;
        let mut known = KnownServers::load(&self.opts.known).map_err(store)?;
        let verifier = Arc::new(Verifier::new(known.get(&key), self.opts.expected, true));
        let (endpoint, conn) = dial(&self.address, &verifier).await?;
        let seen = match verifier.outcome() {
            Some(Ok(seen)) => seen,
            Some(Err(trust)) => return Err(CloseReason::Trust(trust)),
            None => return Err(CloseReason::Lost),
        };
        if seen.first_time {
            known.insert(&key, seen.fingerprint);
            known.save().map_err(store)?;
        }
        Ok((endpoint, conn, seen))
    }

    /// Opens the stream, says hello, then carries traffic until the connection ends.
    async fn live(&self, conn: &Connection, seen: Seen) -> CloseReason {
        let hello = encode(&ClientMsg::Hello(self.opts.hello.clone()));
        let Ok((mut send, recv)) = conn.open_bi().await else {
            return ended(conn.closed().await);
        };
        let Ok(hello) = hello else {
            conn.close(BROKEN, b"no hello");
            return CloseReason::Lost;
        };
        if send.write_all(&hello).await.is_err() {
            return ended(conn.closed().await);
        }
        let _ = self.link.set(conn.clone());
        let connected = ClientEvent::Connected {
            fingerprint: seen.fingerprint,
            first_time: seen.first_time,
        };
        let _ = self.report.send(connected).await;
        let each = async |msg: ServerMsg| self.report.send(ClientEvent::Message(msg)).await.is_ok();
        let reader = link::read(recv, MESSAGE_MAX, each);
        let writer = link::write(conn.clone(), send, self.queue.clone());
        match future::or(writer, future::or(reader, self.datagrams(conn))).await {
            End::Broken => conn.close(BROKEN, b"malformed message"),
            End::Stopped => conn.close(BYE, b"bye"),
            End::Over => {}
        }
        ended(conn.closed().await)
    }

    /// Hands each datagram to the game, dropped when the game is behind.
    async fn datagrams(&self, conn: &Connection) -> End {
        while let Ok(bytes) = conn.read_datagram().await {
            let _ = self.report.try_send(ClientEvent::Datagram(bytes.to_vec()));
        }
        End::Over
    }
}

/// Returns the reason for a list of known servers that cannot be read or written.
fn store(error: io::Error) -> CloseReason {
    CloseReason::Trust(TrustError::Store(error.to_string()))
}
