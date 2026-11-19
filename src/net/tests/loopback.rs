//! Real connections on this machine: joining a room served on the loopback address, and
//! opening a single-player world to LAN and closing it again.

use std::time::{Duration, Instant};

use abstractions_net::io::{ServeConfig, serve};
use abstractions_net::room::{Room, RoomConfig};
use abstractions_net::store::World;
use bevy::prelude::*;

use super::rig::{app, run, scratch};
use crate::computer::Atlas;
use crate::net::host::Hosted;
use crate::net::link::Link;
use crate::net::{CloseLan, JoinServer, LeaveSession, OpenToLan, Session};
use crate::saves::{ActiveWorld, SaveDir, Writer, list, new_entry};
use crate::screens::Screen;
use crate::sim::Changes;
use crate::universe::Universe;
use crate::world::WorldSeed;

/// Longest a test waits for the network.
const PATIENCE: Duration = Duration::from_secs(10);

/// Runs frames until `done` holds, or fails after [`PATIENCE`].
fn until(app: &mut App, what: &str, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + PATIENCE;
    while !done(app) {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        app.update();
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A join through the link reaches a room served on the loopback address and enters its
/// world, with the room's seed.
#[test]
fn a_join_reaches_a_real_room() {
    let config = RoomConfig {
        name: "Loop".to_string(),
        password: None,
        cap: 4,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let world = World {
        seed: 4242,
        ..World::default()
    };
    let at = ServeConfig {
        bind: "127.0.0.1:0".parse().unwrap(),
        dir: scratch("loopback-server"),
    };
    let server = serve(at, Room::new(config, world, 0.0)).unwrap();
    let mut app = app("loopback");
    app.world_mut().write_message(JoinServer {
        address: server.local_addr().to_string(),
        expected_key: Some(server.fingerprint()),
        password: None,
    });
    until(&mut app, "the welcome", |app| {
        matches!(app.world().resource::<Session>(), Session::Joined { .. })
    });
    assert_eq!(app.world().resource::<ActiveWorld>().0.save.seed, 4242);
    assert_eq!(app.world().resource::<WorldSeed>().0, 4242);
    app.world_mut().write_message(LeaveSession);
    run(&mut app, 2);
    assert_eq!(*app.world().resource::<Session>(), Session::Offline);
    server.stop();
}

/// Inserts a single-player world of seed 6 being played, as a loaded world would be.
fn play_alone(app: &mut App) {
    let dir = app.world().resource::<SaveDir>().0.clone();
    let entry = new_entry(&dir, "Isle", 6);
    let world = app.world_mut();
    world.insert_resource(Atlas::new(&Universe::new(6)));
    world.insert_resource(Changes::default());
    world.insert_resource(WorldSeed(6));
    world.insert_resource(ActiveWorld(entry));
    world
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Gameplay);
}

/// Opening to LAN serves the world and joins it without restarting it; closing stops the
/// server and saves the world as in single player.
#[test]
fn a_world_opens_to_lan_and_closes() {
    let mut app = app("host");
    play_alone(&mut app);
    run(&mut app, 2);
    app.world_mut().write_message(OpenToLan);
    until(&mut app, "the own welcome", |app| {
        let link = app.world().get_resource::<Link>();
        link.is_some_and(|link| link.you.is_some())
    });
    assert!(matches!(
        app.world().resource::<Session>(),
        Session::Hosting { .. }
    ));
    assert_eq!(app.world().resource::<ActiveWorld>().0.save.name, "Isle");
    app.world_mut().write_message(CloseLan);
    until(&mut app, "the save", |app| {
        let writer = app.world().resource::<Writer>();
        !app.world().contains_resource::<Hosted>() && !writer.busy()
    });
    assert_eq!(*app.world().resource::<Session>(), Session::Offline);
    assert!(!app.world().contains_resource::<Link>());
    let saved = list(&app.world().resource::<SaveDir>().0);
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].save.seed, 6);
}
