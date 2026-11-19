//! Tests of the length-prefixed stream: whole, cut, joined, oversized and malformed messages.

use super::{
    CLIENT_MESSAGE_MAX, ClientMsg, MESSAGE_MAX, PlayerRecord, StreamError, decode, encode,
};

/// Returns a ping as it sits on a stream.
fn ping(id: u32) -> Vec<u8> {
    encode(&ClientMsg::Ping { id, sent: 0.5 }).unwrap()
}

/// A buffer cut anywhere before the end of a message gives nothing yet, and no error.
#[test]
fn a_cut_message_waits_for_more() {
    let bytes = ping(300);
    for cut in 0..bytes.len() {
        let got = decode::<ClientMsg>(&bytes[..cut], CLIENT_MESSAGE_MAX);
        assert_eq!(got, Ok(None), "cut at {cut}");
    }
}

/// Two messages in one buffer are read one after the other by the bytes each used.
#[test]
fn joined_messages_are_read_in_turn() {
    let mut buffer = ping(1);
    buffer.extend(ping(2));
    buffer.extend(&ping(3)[..3]);
    let (first, used) = decode::<ClientMsg>(&buffer, CLIENT_MESSAGE_MAX)
        .unwrap()
        .unwrap();
    let rest = &buffer[used..];
    let (second, more) = decode::<ClientMsg>(rest, CLIENT_MESSAGE_MAX)
        .unwrap()
        .unwrap();
    assert_eq!(first, ClientMsg::Ping { id: 1, sent: 0.5 });
    assert_eq!(second, ClientMsg::Ping { id: 2, sent: 0.5 });
    assert_eq!(
        decode::<ClientMsg>(&rest[more..], CLIENT_MESSAGE_MAX),
        Ok(None)
    );
}

/// A message that announces more than the cap is refused from its length alone.
#[test]
fn an_oversized_message_is_refused() {
    let announced = (CLIENT_MESSAGE_MAX as u32 + 1).to_le_bytes();
    let got = decode::<ClientMsg>(&announced, CLIENT_MESSAGE_MAX);
    assert_eq!(got, Err(StreamError::TooLarge));
    let exact = (CLIENT_MESSAGE_MAX as u32).to_le_bytes();
    assert_eq!(decode::<ClientMsg>(&exact, CLIENT_MESSAGE_MAX), Ok(None));
    let record = PlayerRecord {
        place: "x".repeat(MESSAGE_MAX),
        ..PlayerRecord::default()
    };
    assert_eq!(
        encode(&ClientMsg::Record(record)),
        Err(StreamError::TooLarge)
    );
}

/// Bytes that are not a message of the expected type are an error, not a panic.
#[test]
fn a_malformed_message_is_an_error() {
    let unknown = [1, 0, 0, 0, 200];
    let got = decode::<ClientMsg>(&unknown, CLIENT_MESSAGE_MAX);
    assert_eq!(got, Err(StreamError::Malformed));
    let empty = [0, 0, 0, 0];
    assert_eq!(
        decode::<ClientMsg>(&empty, CLIENT_MESSAGE_MAX),
        Err(StreamError::Malformed)
    );
    assert_eq!(
        StreamError::TooLarge.to_string(),
        "the message is too large"
    );
}
