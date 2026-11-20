//! The end of a join: the password prompt after a wrong password, and the session that ends on a
//! lost connection.

use abstractions_net::io::{ClientEvent, CloseReason};
use abstractions_net::wire::{Reason, ServerMsg};

use super::kept::{kept, watch};
use super::rig::{ADDRESS, app, push, run, scripted};
use crate::net::link::Link;
use crate::net::{PasswordNeeded, Session, SessionEnded};

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
