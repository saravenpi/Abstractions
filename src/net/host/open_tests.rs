//! Tests of opening a world to LAN: the room's world carries what the player explored, and
//! the room takes as many players as the settings say.

use abstractions_net::store::World;
use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::{Opening, host_cap};
use crate::map::explored::Explored;
use crate::net::{NetDir, Session};
use crate::saves::{ActiveWorld, WorldEntry, WorldSave};
use crate::settings::Settings;
use crate::space::Clock;
use crate::universe::Universe;

/// The world a host serves holds the ground it explored, runs of leaves the room keeps.
#[test]
fn hosting_shares_the_explored_ground() {
    let body = Universe::new(6).home();
    let mut explored = Explored::default();
    explored.reveal(body, DVec3::X, 0.0001, 6.371e6);
    let runs = explored.leaves(body).expect("explored").runs();
    let save = WorldSave {
        seed: 6,
        ..WorldSave::default()
    };
    let mut world = bevy::prelude::World::new();
    world.insert_resource(ActiveWorld(WorldEntry {
        id: "isle".to_string(),
        save,
    }));
    world.insert_resource(explored);
    world.insert_resource(NetDir(std::env::temp_dir()));
    world.init_resource::<Session>();
    world.init_resource::<Clock>();
    let served: Option<World> = world
        .run_system_once(|opening: Opening| opening.world())
        .expect("runs");
    let served = served.expect("a world is played");
    assert_eq!(served.explored, vec![(body, runs)]);
}

/// A world opened to LAN takes as many players as the settings say, 16 by default, within
/// the limit of any server.
#[test]
fn the_host_cap_comes_from_the_settings() {
    assert_eq!(host_cap(None), 16);
    assert_eq!(host_cap(Some(&Settings::default())), 16);
    let cap = |host_max_players| {
        host_cap(Some(&Settings {
            host_max_players,
            ..Settings::default()
        }))
    };
    assert_eq!(cap(100), 100);
    assert_eq!(cap(0), 1);
    assert_eq!(cap(u16::MAX), 1000);
}
