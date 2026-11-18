//! Asking a server what it is, for the server list and `abstractions-server ping`.

use std::sync::Arc;
use std::time::Duration;

use quinn::Connection;
use smol::{Timer, future};

use super::CloseReason;
use super::dial::{dial, ended};
use crate::io::close::BYE;
use crate::io::link::{self, linger};
use crate::trust::Verifier;
use crate::wire::{ClientMsg, MESSAGE_MAX, ServerMsg, Status, encode};

/// Asks the server at `address` for its status and blocks until it answers or `timeout`
/// passes. Returns the status, the round trip time and the server's fingerprint.
///
/// A changed key is not refused here: compare the fingerprint with the known one to show it.
pub fn status(
    address: &str,
    timeout: Duration,
) -> Result<(Status, Duration, [u8; 32]), CloseReason> {
    let late = async {
        Timer::after(timeout).await;
        Err(CloseReason::Timeout)
    };
    smol::block_on(future::or(ask(address), late))
}

/// Connects, asks, and closes.
async fn ask(address: &str) -> Result<(Status, Duration, [u8; 32]), CloseReason> {
    let verifier = Arc::new(Verifier::new(None, None, false));
    let (endpoint, conn) = dial(address, &verifier).await?;
    let answer = question(&conn).await;
    conn.close(BYE, b"bye");
    future::or(endpoint.wait_idle(), linger()).await;
    let fingerprint = verifier.fingerprint().ok_or(CloseReason::Lost)?;
    answer.map(|(status, rtt)| (status, rtt, fingerprint))
}

/// Sends the status request on a new stream and reads the one answer.
async fn question(conn: &Connection) -> Result<(Status, Duration), CloseReason> {
    let (mut send, recv) = conn.open_bi().await.map_err(ended)?;
    let request = encode(&ClientMsg::Status).map_err(|_| CloseReason::Lost)?;
    send.write_all(&request)
        .await
        .map_err(|_| CloseReason::Lost)?;
    let mut answer = None;
    let first = async |msg: ServerMsg| {
        answer = Some(msg);
        false
    };
    link::read(recv, MESSAGE_MAX, first).await;
    match answer {
        Some(ServerMsg::Status(status)) => Ok((status, conn.rtt())),
        _ => Err(conn.close_reason().map_or(CloseReason::Lost, ended)),
    }
}
