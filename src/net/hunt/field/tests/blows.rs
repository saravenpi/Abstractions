//! The blows between robots: the blow a victim takes in and the swing that reaches a body.

use abstractions_net::log::PlayerId as Wired;
use abstractions_net::wire::Blow as Told;
use bevy::prelude::*;

use super::super::duel::reach;
use super::rig::{app, written};
use crate::avatar::Avatar;
use crate::avatar::blow::Blow;
use crate::net::hunt::Echoes;
use crate::player::Player;
use crate::sim::PlayerId;

/// A blow told to the victim becomes the blow of an animal in render space, from the body of the
/// robot, in the frame whose Y is the local up.
#[test]
fn a_blow_on_this_robot_is_written_as_a_blow() {
    let (mut app, _) = app();
    let player = app
        .world_mut()
        .spawn((Player, PlayerId::LOCAL, Transform::from_xyz(0.0, 10.0, 0.0)))
        .id();
    app.world_mut().spawn((Avatar, ChildOf(player)));
    let told = Told {
        victim: Wired(3),
        at: [0.0, 0.5, 0.2],
        joules: 30.0,
        cut: true,
        dir: [0.0, 0.0, -1.0],
        momentum: 6.0,
    };
    app.world_mut()
        .resource_mut::<Echoes>()
        .blows
        .push((Wired(1), told));
    app.update();
    let [blow] = written::<Blow>(&app)[..] else {
        panic!("one blow");
    };
    assert_eq!((blow.joules, blow.cut, blow.momentum), (30.0, true, 6.0));
    assert!(blow.at.distance(Vec3::new(0.0, 10.5, 0.2)) < 1e-3);
    assert!(blow.dir.distance(Vec3::NEG_Z) < 1e-3);
}

/// A blade enters the body of a robot where it passes nearest to its middle, and misses it past the
/// radius of the body.
#[test]
fn a_blade_reaches_the_body_it_passes_through() {
    let blade = [Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, 0.5)];
    let hit = reach(blade, Vec3::new(0.3, 0.0, 0.0));
    assert!(hit.is_some_and(|at| at.distance(Vec3::new(0.0, 0.0, 0.5)) < 1e-4));
    assert!(reach(blade, Vec3::new(2.0, 0.0, 0.0)).is_none());
}
