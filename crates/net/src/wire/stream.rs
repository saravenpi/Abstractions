//! Messages on a stream: each one follows its length, a `u32` in little-endian order.

use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Most bytes of one message, without its length: 4 MiB. Use it as the cap for what a server
/// sends, such as the welcome with the whole log and atlas.
pub const MESSAGE_MAX: usize = 4 << 20;

/// Most bytes a server should take in one message from a player: 64 KiB.
pub const CLIENT_MESSAGE_MAX: usize = 64 << 10;

/// Bytes of the length before each message.
const PREFIX: usize = 4;

/// Why a message could not be written or read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamError {
    /// The message is longer than the cap. Close the stream: what follows cannot be trusted.
    TooLarge,
    /// The bytes are not a message of the expected type.
    Malformed,
}

impl fmt::Display for StreamError {
    /// Writes the error in plain words.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "the message is too large",
            Self::Malformed => "the message is malformed",
        })
    }
}

impl std::error::Error for StreamError {}

/// Returns `message` as bytes ready for a stream: its length, then the message. A message
/// longer than [`MESSAGE_MAX`] is refused.
pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, StreamError> {
    let start = vec![0; PREFIX];
    let mut out = postcard::to_extend(message, start).map_err(|_| StreamError::Malformed)?;
    let size = out.len() - PREFIX;
    if size > MESSAGE_MAX {
        return Err(StreamError::TooLarge);
    }
    out[..PREFIX].copy_from_slice(&(size as u32).to_le_bytes());
    Ok(out)
}

/// Reads one message from the start of `buffer`, which holds what a stream gave so far.
///
/// Returns the message and how many bytes it used: drop those and call again. Returns nothing
/// when the buffer does not hold a whole message yet: read more and call again. A message that
/// announces more than `cap` bytes is refused before it is read.
pub fn decode<T: DeserializeOwned>(
    buffer: &[u8],
    cap: usize,
) -> Result<Option<(T, usize)>, StreamError> {
    let Some((prefix, rest)) = buffer.split_first_chunk::<PREFIX>() else {
        return Ok(None);
    };
    let size = u32::from_le_bytes(*prefix) as usize;
    if size > cap {
        return Err(StreamError::TooLarge);
    }
    let Some(body) = rest.get(..size) else {
        return Ok(None);
    };
    let message = postcard::from_bytes(body).map_err(|_| StreamError::Malformed)?;
    Ok(Some((message, PREFIX + size)))
}
