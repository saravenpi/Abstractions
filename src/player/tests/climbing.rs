//! The rig of the step-up tests: the stage with boxes and stairs on its ground, and a walk.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::{Motion, drive};
use crate::player::eyes::aim;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::view::Target;

/// Height of the flat ground of the stage.
pub(super) const FLOOR: f32 = 10.5;

/// Height of a body centre above the soles.
pub(super) const SOLES: f32 = BODY_LENGTH * 0.5 + BODY_RADIUS;

/// Rise of one stair step, as `world::structures::steps` builds them at most.
pub(super) const RISE: f32 = 0.29;

/// Depth of one stair step, as `world::structures::steps` builds them.
pub(super) const RUN: f32 = 0.36;

/// Returns a stage with one body at rest on the ground, moved by intents, followed by the
/// camera, stepped at `fps` frames a second.
pub(super) fn stage(fps: f64) -> App {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_systems(Update, (gather, drive, aim).chain());
    let frame = Duration::from_secs_f64(1.0 / fps);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(frame));
    app
}

/// Adds a solid box standing on the ground across the path, from `near` to `far` metres ahead
/// of the body, `top` metres high.
pub(super) fn block(app: &mut App, (near, far): (f32, f32), top: f32) {
    let size = Vec3::new(4.0, top + 1.0, far - near);
    let centre = Vec3::new(0.0, FLOOR + top - size.y * 0.5, -(near + far) * 0.5);
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(size.x, size.y, size.z),
        Transform::from_translation(centre),
    ));
}

/// Adds a stair of `steps` steps of [`RISE`] by [`RUN`] going up ahead of the body from
/// `near` metres away, the way `world::structures::steps` lays its boxes: one column per step
/// from the ground up. The top step goes on for 6 metres.
pub(super) fn stair(app: &mut App, near: f32, steps: usize) {
    for index in 0..steps {
        let from = near + RUN * index as f32;
        let to = from + RUN + 6.0 * f32::from(u8::from(index + 1 == steps));
        block(app, (from, to), RISE * (index + 1) as f32);
    }
}

/// Moves the body to `at` at rest.
pub(super) fn put(app: &mut App, at: Vec3) {
    let world = app.world_mut();
    let mut bodies = world.query_filtered::<(&mut Transform, &mut Motion), With<Player>>();
    for (mut transform, mut motion) in bodies.iter_mut(world) {
        transform.translation = at;
        *motion = Motion::default();
    }
}

/// Walks the body `forward` (1 ahead, -1 back) for `frames` frames and returns, for each
/// frame, its pose, its motion and where the camera rig is told the body is.
pub(super) fn walk(app: &mut App, forward: f32, frames: usize) -> Vec<(Transform, Motion, Vec3)> {
    let mut seen = Vec::with_capacity(frames);
    for _ in 0..frames {
        let player = PlayerId::LOCAL;
        let moving = Intent::Move {
            forward,
            strafe: 0.0,
        };
        app.world_mut().write_message(IntentOf {
            player,
            intent: moving,
        });
        app.update();
        let (transform, motion, _) = body(app, player);
        seen.push((transform, motion, app.world().resource::<Target>().position));
    }
    seen
}

/// Returns the height of the soles of a body posed at `transform` above the ground.
pub(super) fn soles(transform: &Transform) -> f32 {
    transform.translation.y - SOLES - FLOOR
}
