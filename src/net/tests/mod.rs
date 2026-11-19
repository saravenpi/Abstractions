//! Session tests: which sessions count as shared, then the network driver in the modules
//! below, against a scripted server and against real ones on this machine.

mod end;
mod join;
mod kept;
mod lobby;
mod loopback;
mod rig;
mod shared;
mod single;
mod world;

use super::Session;

/// Joined and hosting share the universe; offline and joining do not.
#[test]
fn only_joined_and_hosting_are_shared() {
    let joining = Session::Joining {
        address: "space.facile.studio".to_string(),
    };
    let joined = Session::Joined {
        address: "space.facile.studio".to_string(),
        name: "Space".to_string(),
    };
    let hosting = Session::Hosting {
        lan_address: "192.168.1.20:24650".to_string(),
        key: "7F3A-91C2-0B5E".to_string(),
    };
    assert!(!Session::Offline.shared());
    assert!(!joining.shared());
    assert!(joined.shared() && hosting.shared());
    assert_eq!(Session::default(), Session::Offline);
}
