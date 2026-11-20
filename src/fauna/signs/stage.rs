//! The stage of the signs tests: the fauna rig on the home planet of seed 1, with the lay and the
//! hearing added after the scan, a camera at the robot, and a count of the calls sent each frame.

use bevy::prelude::*;

use super::{Trail, hear, lay};
use crate::audio::AudioCue;
use crate::fauna::herds::Herds;
use crate::life::herd::Herd;
use crate::render::GameCamera;

/// Calls sent to the audio in each frame so far.
#[derive(Resource, Default)]
pub struct Sent(pub Vec<usize>);

/// Counts the calls sent this frame.
fn count(mut cues: MessageReader<AudioCue>, mut sent: ResMut<Sent>) {
    let calls = cues
        .read()
        .filter(|c| matches!(c, AudioCue::Call(_)))
        .count();
    sent.0.push(calls);
}

/// Returns the rig app, on a living or dead home planet, with the signs systems, and the herd the robot
/// stands in when the planet has one.
pub fn app(alive: bool) -> (App, Option<Herd>) {
    let (mut app, herd) = crate::fauna::rig::app(alive);
    app.init_resource::<Trail>()
        .init_resource::<Sent>()
        .add_message::<AudioCue>()
        .add_systems(
            Update,
            (lay::forget, lay::lay, hear::hear, count)
                .chain()
                .after(crate::fauna::fade::fade),
        );
    let world = app.world_mut();
    let mut robot = world.query_filtered::<&Transform, With<crate::player::Player>>();
    let at = robot.single(world).map_or(Transform::IDENTITY, |t| *t);
    world.spawn((GameCamera, GlobalTransform::from(at)));
    (app, herd)
}

/// Returns true when the herds of the rig hold any herd at all.
pub fn herded(app: &App) -> bool {
    !app.world().resource::<Herds>().herds.is_empty()
}
