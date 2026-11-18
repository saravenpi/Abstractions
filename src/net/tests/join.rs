//! The join flow against a scripted server: the welcome that starts the server's world, and
//! leaving with the record and the goodbye.

use abstractions_net::io::ClientEvent;
use abstractions_net::log::{Change, Changes, PlayerId};
use abstractions_net::wire::{ClientMsg, PlayerRecord, ServerMsg};
use bevy::prelude::*;

use super::kept::{kept, watch};
use super::rig::{ADDRESS, app, push, run, scripted, sent, welcome};
use crate::computer::Atlas;
use crate::net::join::world::SESSION_ID;
use crate::net::link::{FreshKey, Link};
use crate::net::{LeaveSession, Lobby, Players, RowStatus, ServerRow, Session, SessionEnded};
use crate::saves::{ActiveWorld, Place, Unsaved};
use crate::space::Clock;
use crate::universe::{BodyId, Universe};
use crate::world::WorldSeed;

/// Returns a moon of the home planet of `seed`, a body a new game has not visited.
pub fn moon(seed: u64, moon: u8) -> BodyId {
    BodyId {
        moon,
        ..Universe::new(seed).home()
    }
}

/// Returns a log holding a first visit of `body` by player 2.
pub fn visited(body: BodyId) -> Changes {
    let mut changes = Changes::default();
    changes.push(PlayerId(2), 40.0, Change::Visited(body));
    changes
}

/// Returns the place the scripted server kept of the player: on the first moon of seed 77.
fn kept_place() -> Place {
    Place {
        body: moon(77, 1),
        dir: [0.0, 1.0, 0.0],
        altitude: 3.0,
    }
}

/// Returns an app joined to a scripted server named "Friends" whose welcome keeps the place
/// of [`kept_place`] and logs a visit of the second moon, a server seen the first time.
fn welcomed(tag: &str) -> App {
    let mut app = app(tag);
    app.world_mut().resource_mut::<Lobby>().saved = vec![ServerRow {
        name: "Friends".to_string(),
        address: ADDRESS.to_string(),
        status: RowStatus::Pinging,
        key: None,
    }];
    let script = scripted(&mut app);
    let record = PlayerRecord {
        place: toml::to_string(&kept_place()).unwrap(),
        ship: String::new(),
    };
    let first = ClientEvent::Connected {
        fingerprint: [0x7f; 32],
        first_time: true,
    };
    push(&script, first);
    let hello = welcome(77, visited(moon(77, 2)), Some(record));
    push(&script, ClientEvent::Message(ServerMsg::Welcome(hello)));
    run(&mut app, 3);
    app
}

/// A welcome starts the server's world in memory: its seed, clock, log and atlas, the place
/// the server kept, and the name the server gives itself.
#[test]
fn a_welcome_starts_the_world_of_the_server() {
    let app = welcomed("welcome");
    let world = app.world();
    let joined = Session::Joined {
        address: ADDRESS.to_string(),
        name: "Friends".to_string(),
    };
    assert_eq!(*world.resource::<Session>(), joined);
    let active = &world.resource::<ActiveWorld>().0;
    assert_eq!((active.id.as_str(), active.save.seed), (SESSION_ID, 77));
    assert_eq!(active.save.place, Some(kept_place()));
    assert!(world.contains_resource::<Unsaved>());
    assert_eq!(world.resource::<WorldSeed>().0, 77);
    assert_eq!(world.resource::<Clock>().seconds, 1234.5);
    assert_eq!(*world.resource::<Changes>(), visited(moon(77, 2)));
    let bodies = &world.resource::<Atlas>().bodies;
    assert_eq!(*bodies, [Universe::new(77).home(), moon(77, 2)]);
}

/// A welcome also gives the player's id and the players already there, and leaves the key of
/// a server seen the first time for the banner.
#[test]
fn a_welcome_names_the_players_and_the_new_key() {
    let app = welcomed("players");
    let world = app.world();
    assert_eq!(world.resource::<Players>().0, [(1, "Lin".to_string())]);
    assert_eq!(world.resource::<Link>().you, Some(PlayerId(3)));
    let fresh = world.resource::<FreshKey>().0.clone();
    assert_eq!(fresh.as_deref(), Some("New server key 7F7F-7F7F-7F7F"));
}

/// Leaving sends the place and ship, says goodbye and goes back to single player, with no
/// reason to show.
#[test]
fn leaving_sends_the_record_and_the_goodbye() {
    let mut app = app("leave");
    watch::<SessionEnded>(&mut app);
    let script = scripted(&mut app);
    let hello = welcome(5, Changes::default(), None);
    push(&script, ClientEvent::Message(ServerMsg::Welcome(hello)));
    run(&mut app, 2);
    app.world_mut().write_message(LeaveSession);
    run(&mut app, 2);
    let sent = sent(&script);
    assert!(matches!(
        sent[..],
        [.., ClientMsg::Record(_), ClientMsg::Bye]
    ));
    assert_eq!(*app.world().resource::<Session>(), Session::Offline);
    assert!(!app.world().contains_resource::<Link>());
    assert!(kept::<SessionEnded>(&app).is_empty());
}
