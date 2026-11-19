//! The end of a join: refusals and lost connections said in plain words, and the password
//! prompt after a wrong password.

use abstractions_net::io::{AddrError, ClientEvent, CloseReason};
use abstractions_net::trust::TrustError;
use abstractions_net::wire::{Reason, ServerMsg};

use super::kept::{kept, watch};
use super::rig::{ADDRESS, app, push, run, scripted};
use crate::net::join::end::words;
use crate::net::link::Link;
use crate::net::{PasswordNeeded, Session, SessionEnded};

/// Every reason reads as a short sentence; the version the server runs names what is needed.
#[test]
fn every_reason_has_plain_words() {
    let said = |reason: CloseReason| words(&reason, "home.example", Some("0.9.0"));
    let refused = |reason| said(CloseReason::Refused(reason));
    assert_eq!(refused(Reason::WrongPassword), "Wrong password");
    assert_eq!(refused(Reason::Full), "The server is full");
    assert_eq!(refused(Reason::Outdated), "This server needs v0.9.0");
    assert_eq!(refused(Reason::Newer), "You need v0.9.0");
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
    let unknown = words(&CloseReason::Refused(Reason::Outdated), "x", None);
    assert_eq!(unknown, "This server needs a newer game");
}

/// A refused password ends the session with its reason and asks for a password once, even
/// when the close follows the refusal in the same frame.
#[test]
fn a_wrong_password_ends_the_join_and_asks_for_one() {
    let mut app = app("password");
    watch::<SessionEnded>(&mut app);
    watch::<PasswordNeeded>(&mut app);
    let script = scripted(&mut app);
    let refused = Reason::WrongPassword;
    push(&script, ClientEvent::Message(ServerMsg::Refused(refused)));
    push(&script, ClientEvent::Closed(CloseReason::Refused(refused)));
    run(&mut app, 2);
    assert_eq!(*app.world().resource::<Session>(), Session::Offline);
    assert!(!app.world().contains_resource::<Link>());
    let ended: Vec<String> = kept::<SessionEnded>(&app)
        .into_iter()
        .map(|end| end.reason)
        .collect();
    assert_eq!(ended, ["Wrong password"]);
    let asked = kept::<PasswordNeeded>(&app);
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].address, ADDRESS);
}

/// A server that never answers ends the join with its address.
#[test]
fn an_unreachable_server_ends_the_join() {
    let mut app = app("unreachable");
    watch::<SessionEnded>(&mut app);
    let script = scripted(&mut app);
    push(&script, ClientEvent::Closed(CloseReason::Unreachable));
    run(&mut app, 2);
    let ended = kept::<SessionEnded>(&app);
    assert_eq!(ended[0].reason, format!("No answer from {ADDRESS}"));
    assert!(kept::<SessionEnded>(&app).len() == 1);
}
