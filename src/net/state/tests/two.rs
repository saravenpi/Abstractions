//! Two players in one world: the body of the player at this machine and the puppet of another, as
//! the session draws it. The game finds "the robot of this machine" with `With<Player>`, and many
//! systems ask for exactly one, so a puppet must never carry that marker nor hang from a body that
//! does.

use abstractions_net::io::ClientEvent;
use abstractions_net::wire::Frame;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::stage::{app, frame_at, ground, linked, puppets, robot};
use crate::avatar::Avatar;
use crate::net::Remote;
use crate::player::Player;
use crate::sim::PlayerId;

/// Universe time the records start at.
const START: f64 = 500.0;

/// With the local body in the world and a second player drawn from two seconds of its records,
/// exactly one entity is a `Player`, the query the combat, hunting, craft and guide systems take
/// their one robot from finds it, and the puppet is a parentless `Avatar` marked `Remote`.
#[test]
fn a_second_player_is_a_puppet_and_never_a_player() {
    let mut app = app("two");
    let script = linked(&mut app);
    let (home, spot) = ground();
    let local = app
        .world_mut()
        .spawn((Player, Transform::from_xyz(1.0, 1.0, 1.0)))
        .id();
    for step in 0..60 {
        let time = START + f64::from(step) / 30.0;
        let batch = robot(PlayerId(7), Frame::Surface(home), time, (spot, DVec3::ZERO));
        let event = ClientEvent::Datagram(batch);
        script.lock().unwrap().events.push_back(event);
        frame_at(&mut app, time);
    }
    assert_eq!(puppets(&mut app).len(), 1, "the second player is not drawn");
    let world = app.world_mut();
    let mut players = world.query_filtered::<Entity, With<Player>>();
    assert_eq!(players.single(world).ok(), Some(local));
    let mut marked = world.query_filtered::<(), (With<Remote>, With<Player>)>();
    assert_eq!(marked.iter(world).count(), 0, "a puppet carries Player");
    let mut bodies = world.query_filtered::<Option<&ChildOf>, (With<Avatar>, With<Remote>)>();
    let parents: Vec<Option<Entity>> = bodies
        .iter(world)
        .map(|of| of.map(ChildOf::parent))
        .collect();
    assert_eq!(parents, [None], "the robot puppet hangs from something");
}
