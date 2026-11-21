//! Tests of the knap: the pick key has the robot wind up and strike the rock, stooping to a low one,
//! and the piece comes away once, at the moment the blow reaches the rock, not at the key press.

use std::time::Duration;

use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::super::super::Avatar;
use super::super::layer::Layer;
use super::knap::{self, Knap, struck};
use super::tune::KNAP;
use crate::craft::build::Hands;
use crate::craft::pack::{Knock, Knocked};
use crate::player::Player;
use crate::sim::PlayerId;
use crate::world::rocks::{Plant, Rock, RockId};

/// Frames per second of the test app.
const FPS: f32 = 60.0;

/// Where the rock is struck, in render space: low and ahead of the robot at the origin.
const LOW_ROCK: Vec3 = Vec3::new(0.15, -0.6, -1.0);

/// Returns a rock of the test.
fn rock() -> Rock {
    Rock {
        id: RockId(9),
        plant: Plant::Rock,
        scale: 1.0,
        at: Vec3::ZERO,
        mid: Vec3::Y * 0.45,
        left: 1.0,
    }
}

/// The knap stoops the body to a low rock and has it under way through the blow; with a tool the
/// blow is overhead, with the fist it is a punch, and both are laid in full.
#[test]
fn the_knap_strikes_down_at_the_rock() {
    let blow = KNAP[0] + 0.5 * KNAP[1];
    let fist = struck((LOW_ROCK, blow), None, 1.0);
    let tool = struck((LOW_ROCK, blow), Some(Hands::Two), 1.0);
    for layer in [fist, tool] {
        assert_eq!(layer.weight, 1.0);
        assert!(layer.legs > 0.9 && layer.stance.lift < -0.2, "{layer:?}");
    }
    assert!(fist.free.is_some() && tool.free.is_none());
    let start = struck((LOW_ROCK, 0.0), None, 1.0);
    assert!(start.legs < 0.05);
}

/// Returns an app that runs the knap at a steady frame rate, with the local robot standing at the
/// origin, and its body.
fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / FPS,
        )))
        .add_message::<Knock>()
        .add_message::<Knocked>()
        .add_systems(Update, (knap::start, knap::knap).chain());
    let player = app.world_mut().spawn((Player, PlayerId::LOCAL)).id();
    let body = app
        .world_mut()
        .spawn((Avatar, GlobalTransform::default(), ChildOf(player)))
        .id();
    (app, body)
}

/// A knock asked of the local robot plays the knap, says the blow reached the rock once, at the
/// contact, and ends with the layer let go.
#[test]
fn the_piece_comes_away_at_the_contact() {
    let (mut app, body) = app();
    app.world_mut().write_message(Knock {
        rock: rock(),
        at: LOW_ROCK,
    });
    let mut came = Vec::new();
    for frame in 0..(FPS as usize) {
        app.update();
        let told = app
            .world_mut()
            .resource_mut::<Messages<Knocked>>()
            .drain()
            .count();
        came.extend(std::iter::repeat_n(frame, told));
    }
    assert_eq!(came.len(), 1, "{came:?}");
    let contact = ((KNAP[0] + KNAP[1]) * FPS) as usize;
    assert!(
        came[0] + 3 >= contact && came[0] <= contact + 3,
        "{came:?} {contact}"
    );
    assert!(app.world().get::<Knap>(body).is_none());
    let layer = app.world().get::<Layer>(body).copied().unwrap_or_default();
    assert_eq!(layer.weight, 0.0);
}
