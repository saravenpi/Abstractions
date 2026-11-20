//! The reliable stream of a connection, read and written as whole messages.

use std::time::Duration;

use quinn::{Connection, RecvStream, SendStream, VarInt};
use serde::de::DeserializeOwned;
use smol::channel::Receiver;
use smol::future;

use super::close::BYE;
use crate::wire::{StreamError, decode};

/// How long a closing side waits for the other to read what is left on the stream.
const LINGER: Duration = Duration::from_secs(1);

/// Something to do on the stream, queued for its writer.
#[derive(Debug)]
pub(crate) enum Out {
    /// Write these bytes: one message as [`crate::wire::encode`] made it.
    Frame(Vec<u8>),
    /// Finish the stream, let the other side read it, then close with this code and text.
    Close(VarInt, Vec<u8>),
}

/// How reading the stream ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum End {
    /// The stream or the connection went away: the connection says why.
    Over,
    /// The stream carried something that is not a message.
    Broken,
    /// The one handling the messages wants no more.
    Stopped,
}

/// Reads whole messages from `recv` and hands each to `each` until it returns false. A message
/// longer than `cap` breaks the stream. A stream the other side finished waits forever: the
/// other side may still be closing, and the connection's end tells why.
pub(crate) async fn read<T: DeserializeOwned>(
    mut recv: RecvStream,
    cap: usize,
    mut each: impl AsyncFnMut(T) -> bool,
) -> End {
    let mut buffer = Vec::new();
    loop {
        match take(&mut buffer, cap) {
            Ok(Some(message)) => {
                if !each(message).await {
                    return End::Stopped;
                }
            }
            Ok(None) => match recv.read_chunk(cap, true).await {
                Ok(Some(chunk)) => buffer.extend_from_slice(&chunk.bytes),
                Ok(None) => future::pending().await,
                Err(_) => return End::Over,
            },
            Err(_) => return End::Broken,
        }
    }
}

/// Takes the first whole message out of `buffer`, if it holds one.
fn take<T: DeserializeOwned>(buffer: &mut Vec<u8>, cap: usize) -> Result<Option<T>, StreamError> {
    let Some((message, used)) = decode(buffer, cap)? else {
        return Ok(None);
    };
    buffer.drain(..used);
    Ok(Some(message))
}

/// Writes what `queue` holds to `send` until a close or the end of the queue, then closes
/// the connection.
pub(crate) async fn write(conn: Connection, mut send: SendStream, queue: Receiver<Out>) -> End {
    while let Ok(out) = queue.recv().await {
        match out {
            Out::Frame(bytes) if send.write_all(&bytes).await.is_ok() => {}
            Out::Frame(_) => return End::Over,
            Out::Close(code, text) => return finish(conn, send, code, &text).await,
        }
    }
    finish(conn, send, BYE, b"bye").await
}

/// Finishes the stream, gives the other side [`LINGER`] to read it, then closes with `code`.
async fn finish(conn: Connection, mut send: SendStream, code: VarInt, text: &[u8]) -> End {
    let _ = send.finish();
    future::or(async { drop(send.stopped().await) }, linger()).await;
    conn.close(code, text);
    End::Stopped
}

/// Waits as long as a closing side gives the other to read the end of the stream.
pub(crate) async fn linger() {
    smol::Timer::after(LINGER).await;
}
