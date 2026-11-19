//! Tests of opening a world to LAN: the room's world carries what the player explored.

use abstractions_net::room::EXPLORED_WORDS;
use abstractions_net::store::World;
use bevy::ecs::system::RunSystemOnce;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Opening;
use crate::map::explored::Explored;
use crate::map::explored::grid::WORDS;
use crate::net::{NetDir, Session};
use crate::saves::{ActiveWorld, WorldEntry, WorldSave};
use crate::space::Clock;
use crate::universe::Universe;

/// The world a host serves holds the ground it explored, whole bitsets the room keeps.
#[test]
fn hosting_shares_the_explored_ground() {
    let body = Universe::new(6).home();
    let mut explored = Explored::default();
    explored.reveal(body, DVec3::X, 0.01);
    let words = explored.words(body).expect("explored").to_vec();
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
    assert_eq!(WORDS, EXPLORED_WORDS);
    assert_eq!(served.explored, vec![(body, words)]);
}
