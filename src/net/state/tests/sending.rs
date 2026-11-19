//! Sending in a headless app: the robot of this machine goes out 30 times a second while it
//! walks and twice a second with the rest flag once it stands, on the surface it stands on,
//! and the batch it is written in is the same buffer every frame.

use std::time::Duration;

use abstractions_net::wire::{Frame, Kind, RobotBody, records};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::stage::{YOU, app, ground, linked};
use crate::avatar::{Avatar, AvatarPose, Mode};
use crate::net::link::Script;
use crate::net::state::send::Outbox;
use crate::player::{Motion, Player};
use crate::sim::PlayerId;

/// Spawns the player of this machine walking at 2 m/s one metre above the anchored spot, with
/// its robot, and returns the player.
fn walker(app: &mut App) -> Entity {
    let motion = Motion {
        velocity: Vec3::X * 2.0,
        ..default()
    };
    let body = (
        Player,
        PlayerId::LOCAL,
        motion,
        GlobalTransform::from_translation(Vec3::Y),
    );
    let player = app.world_mut().spawn(body).id();
    let pose = AvatarPose {
        mode: Mode::Walk,
        ..default()
    };
    app.world_mut().spawn((Avatar, pose, ChildOf(player)));
    player
}

/// Runs `frames` frames and returns the robot records sent meanwhile, with their rest flag.
fn sent(app: &mut App, script: &Script, frames: usize) -> Vec<(bool, RobotBody)> {
    script.lock().unwrap().datagrams.clear();
    for _ in 0..frames {
        app.update();
    }
    let datagrams = script.lock().unwrap().datagrams.clone();
    let all = datagrams.iter().flat_map(|batch| {
        records(batch)
            .map(|record| (record.head, record.body.to_vec()))
            .collect::<Vec<_>>()
    });
    all.filter(|(head, _)| head.kind == Kind::Robot && head.player == YOU)
        .map(|(head, body)| {
            assert_eq!(head.frame, Frame::Surface(ground().0));
            (
                head.at_rest(),
                RobotBody::decode(&head.frame, &body).unwrap(),
            )
        })
        .collect()
}

/// At 120 frames a second the walking robot is sent 30 times a second, the standing one twice
/// with the rest flag, where it is on the planet.
#[test]
fn the_robot_goes_out_at_its_rate() {
    let mut app = app("send");
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 120.0,
    )));
    let script = linked(&mut app);
    let player = walker(&mut app);
    app.update();
    let moving = sent(&mut app, &script, 240);
    assert!(
        (59..=61).contains(&moving.len()),
        "{} records walking",
        moving.len()
    );
    assert!(moving.iter().all(|(rest, body)| !rest && body.mode == 1));
    let (_, spot) = ground();
    assert!((moving[0].1.position.distance(spot) - 1.0).abs() < 0.01);
    app.world_mut().get_mut::<Motion>(player).unwrap().velocity = Vec3::ZERO;
    let mut poses = app.world_mut().query::<&mut AvatarPose>();
    poses.single_mut(app.world_mut()).unwrap().mode = Mode::Idle;
    let resting = sent(&mut app, &script, 240);
    assert!(
        (3..=5).contains(&resting.len()),
        "{} records standing",
        resting.len()
    );
    assert!(resting.iter().all(|(rest, _)| *rest));
}

/// The batch is written into the same buffer every frame: it does not grow or move.
#[test]
fn the_batch_is_reused() {
    let mut app = app("reuse");
    let _script = linked(&mut app);
    walker(&mut app);
    for _ in 0..10 {
        app.update();
    }
    let batch = &app.world().resource::<Outbox>().batch;
    let kept = (batch.as_ptr(), batch.capacity());
    for _ in 0..100 {
        app.update();
    }
    let batch = &app.world().resource::<Outbox>().batch;
    assert_eq!((batch.as_ptr(), batch.capacity()), kept);
}
