//! Test-only stage for touch: one static collider, a ball dropped on it, and the capsule of the player swept
//! into it the way the controller of the player moves, by avian's move and slide.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::player::{BODY_LENGTH, BODY_RADIUS};

/// Radius of the dropped ball, the radius of the player.
pub(crate) const BALL: f32 = BODY_RADIUS;

/// Returns a headless physics app at 30 frames a second that holds `solid` as a static body, with the
/// collider trees already built.
pub(crate) fn stage(solid: Collider) -> App {
    let frame = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 30.0));
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .init_asset::<Mesh>()
        .insert_resource(frame);
    app.finish();
    app.cleanup();
    let body = (RigidBody::Static, solid, Friction::new(1.0));
    app.world_mut().spawn((body, Transform::default()));
    (0..3).for_each(|_| app.update());
    app
}

/// Drops a ball the size of the player from `at`, free to fall but not to roll, runs five seconds and
/// returns where it is and how fast it still moves.
pub(crate) fn dropped(app: &mut App, at: Vec3) -> (Vec3, f32) {
    let ball = (
        RigidBody::Dynamic,
        Collider::sphere(BALL),
        Friction::new(1.0),
    );
    let still = (LockedAxes::ROTATION_LOCKED, Transform::from_translation(at));
    let id = app.world_mut().spawn((ball, still)).id();
    (0..150).for_each(|_| app.update());
    let world = app.world();
    let place = world.get::<Transform>(id).expect("the ball exists");
    let speed = world.get::<LinearVelocity>(id).expect("the ball moves");
    (place.translation, speed.0.length())
}

/// Pushes the upright capsule of the player from `from` at `velocity` for `frames` frames of `rate` frames
/// a second, sliding along what it meets, and returns where it ends.
pub(crate) fn pushed(
    app: &mut App,
    from: Vec3,
    velocity: Vec3,
    (frames, rate): (u32, f32),
) -> Vec3 {
    let sweep = move |slide: MoveAndSlide| {
        let body = Collider::capsule(BODY_RADIUS, BODY_LENGTH);
        let step = Duration::from_secs_f32(1.0 / rate);
        let (config, filter) = (MoveAndSlideConfig::default(), SpatialQueryFilter::default());
        let (rot, turn) = (Quat::IDENTITY, (&config, &filter));
        let moved = |at: Vec3| {
            slide.move_and_slide(&body, at, rot, velocity, step, turn.0, turn.1, |_| {
                MoveAndSlideHitResponse::Accept
            })
        };
        (0..frames).fold(from, |at, _| moved(at).position)
    };
    let end = app.world_mut().run_system_once(sweep);
    end.expect("the sweep runs")
}
