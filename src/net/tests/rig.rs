//! The headless app of the network tests: the session seam, the driver, the saves, the log of
//! first visits and the screen states, with every file in a scratch folder of its own, and a
//! scripted link in place of a server.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use abstractions_net::io::ClientEvent;
use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{ClientMsg, PlayerInfo, PlayerRecord, Welcome};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::menus::Menu;
use crate::net::join::open;
use crate::net::link::{Link, Role, Script, Source, Wire};
use crate::net::{NetDir, Session};
use crate::render::DayClock;
use crate::saves::SaveDir;
use crate::screens::Screen;
use crate::space::Clock;
use crate::view::Heading;

/// The address the scripted server answers on.
pub const ADDRESS: &str = "friends.example:24650";

/// Returns an empty scratch folder of its own for one test.
pub fn scratch(tag: &str) -> PathBuf {
    let name = format!("abstractions-net-{tag}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Returns a headless app on the title screen with the network seam and driver, the saves
/// and the log of first visits, its files in the scratch folder of `tag`.
pub fn app(tag: &str) -> App {
    let dir = scratch(tag);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Screen>()
        .init_state::<Menu>()
        .insert_resource(SaveDir(dir.join("worlds")))
        .insert_resource(NetDir(dir.join("config")))
        .init_resource::<DayClock>()
        .init_resource::<Heading>()
        .init_resource::<Clock>()
        .add_plugins((
            crate::net::plugin,
            crate::net::driver::plugin,
            crate::saves::plugin,
            crate::sim::plugin,
        ));
    app.update();
    app
}

/// Inserts a scripted link joining [`ADDRESS`], as a join would, and returns its script.
pub fn scripted(app: &mut App) -> Script {
    let script: Script = Arc::new(Mutex::new(Wire::default()));
    let role = Role::Join {
        address: ADDRESS.to_string(),
        expected: None,
    };
    let world = app.world_mut();
    open(
        &mut world.commands(),
        Link::over(Source::Script(script.clone()), role),
    );
    world.flush();
    world.insert_resource(Session::Joining {
        address: ADDRESS.to_string(),
    });
    script
}

/// Hands `event` to the link on its next read.
pub fn push(script: &Script, event: ClientEvent) {
    script.lock().unwrap().events.push_back(event);
}

/// Returns the messages the game sent through the link.
pub fn sent(script: &Script) -> Vec<ClientMsg> {
    script.lock().unwrap().sent.clone()
}

/// Returns a welcome to a world of `seed` at universe time 1234.5, with `changes`, an empty
/// atlas, the record `record` and Lin already there.
pub fn welcome(seed: u64, changes: Changes, record: Option<PlayerRecord>) -> Welcome {
    Welcome {
        you: PlayerId(3),
        seed,
        clock: 1234.5,
        changes,
        atlas: String::new(),
        record,
        players: vec![PlayerInfo {
            id: PlayerId(1),
            name: "Lin".to_string(),
        }],
    }
}

/// Runs `frames` frames.
pub fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}
