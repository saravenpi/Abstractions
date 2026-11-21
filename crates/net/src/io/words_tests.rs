//! Tests of the words a connection ends with.

use super::{AddrError, CloseReason};
use crate::trust::TrustError;
use crate::wire::Reason;

/// Every reason reads as a short sentence; the version the server runs names what is needed.
#[test]
fn every_reason_has_plain_words() {
    let said = |reason: CloseReason| reason.words("home.example", Some("0.9.0"));
    let refused = |reason| said(CloseReason::Refused(reason));
    assert_eq!(refused(Reason::WrongPassword), "Wrong password");
    assert_eq!(refused(Reason::Full), "The server is full");
    assert_eq!(refused(Reason::Outdated), "This server needs v0.9.0");
    assert_eq!(refused(Reason::Newer), "You need v0.9.0");
    assert_eq!(
        refused(Reason::Flooding),
        "The server sent you away for sending too much"
    );
    assert_eq!(refused(Reason::Closing), "The server closed");
    let changed = TrustError::Changed {
        expected: [1; 32],
        got: [2; 32],
    };
    assert_eq!(
        said(CloseReason::Trust(changed)),
        "This server's key changed"
    );
    assert_eq!(said(CloseReason::Timeout), "No answer from home.example");
    assert_eq!(
        said(CloseReason::Unreachable),
        "No answer from home.example"
    );
    assert_eq!(said(CloseReason::Lost), "Connection lost");
    let malformed = CloseReason::Address(AddrError::Malformed);
    assert_eq!(said(malformed), "This is not an address");
    let unknown = CloseReason::Refused(Reason::Outdated).words("x", None);
    assert_eq!(unknown, "This server needs a newer game");
}

/// A player sent away for joining again or for going silent reads why.
#[test]
fn a_seat_taken_away_has_plain_words() {
    let refused = |reason| CloseReason::Refused(reason).words("home.example", None);
    assert_eq!(
        refused(Reason::Replaced),
        "You joined this server from another game"
    );
    assert_eq!(
        refused(Reason::Silent),
        "The server stopped hearing from your game"
    );
}
