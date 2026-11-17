//! The systems inside a headless app: nothing moves until the rig is enabled, then the game camera follows.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use super::third;
use crate::menus::Menu;
use crate::player::Player;
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::view::{BodyFade, CameraPose, Cut, Enabled, Heading, Target, ViewPrefs};

/// Returns default settings stored in a scratch file of their own.
fn settings() -> Persistent<Settings> {
    let dir = format!("abstractions-view-test-{}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(dir).join("settings.toml"))
        .default(Settings::default())
        .build()
        .unwrap()
}

/// Builds a headless game in play with the view plugin, in third person.
fn engine() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, TransformPlugin, InputPlugin))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )))
        .insert_resource(settings())
        .init_state::<Screen>()
        .init_state::<Menu>()
        .add_plugins(crate::view::plugin)
        .insert_state(Screen::Gameplay)
        .insert_resource(third());
    app.finish();
    app.cleanup();
    app
}

/// Puts a wall behind a turned player body and a camera that is a child of that body in a
/// game. Returns the game and the camera.
fn game() -> (App, Entity) {
    let mut app = engine();
    let body = Transform::from_xyz(4.0, 0.9, 2.0).with_rotation(Quat::from_rotation_y(0.5));
    let world = app.world_mut();
    let wall = Transform::from_xyz(4.0, 2.0, 4.6);
    world.spawn((RigidBody::Static, Collider::cuboid(8.0, 4.0, 0.2), wall));
    let solid = (RigidBody::Kinematic, Collider::capsule(0.35, 1.1));
    let player = world.spawn((Player, body, solid)).id();
    let eye = Transform::from_xyz(0.0, 0.7, 0.0);
    let camera = (GameCamera, eye, Projection::default(), ChildOf(player));
    let camera = world.spawn(camera).id();
    world.resource_mut::<Target>().position = body.translation;
    (app, camera)
}

/// Returns where the camera is in render space after the transforms were propagated.
fn camera_at(app: &App, camera: Entity) -> Vec3 {
    app.world()
        .get::<GlobalTransform>(camera)
        .unwrap()
        .translation()
}

/// Runs `frames` frames of the game.
fn play(app: &mut App, frames: usize) {
    (0..frames).for_each(|_| app.update());
}

/// The rig leaves the camera alone until it is enabled.
#[test]
fn systems_wait_for_enabled() {
    let (mut app, camera) = game();
    play(&mut app, 5);
    assert!(camera_at(&app, camera).distance(Vec3::new(4.0, 1.6, 2.0)) < 1.0e-4);
    assert_eq!(*app.world().resource::<CameraPose>(), CameraPose::default());
}

/// Once enabled, the rig puts the camera behind the player in render space although the
/// camera is a child of the turned body, keeps it in front of the wall, looks through the
/// body of the player and honours a cut.
#[test]
fn systems_drive_the_game_camera() {
    let (mut app, camera) = game();
    app.insert_resource(Enabled(true));
    play(&mut app, 30);
    let pose = *app.world().resource::<CameraPose>();
    let at = camera_at(&app, camera);
    assert!(
        at.distance(pose.position) < 1.0e-4,
        "the camera is at {at}, not at {pose:?}"
    );
    assert!(
        at.z > 3.5 && at.z < 4.5 - 0.3 + 1.0e-3,
        "the wall did not stop the camera: {at}"
    );
    assert_eq!(app.world().resource::<BodyFade>().0, 1.0);
    let heading = Heading {
        yaw: 1.2,
        pitch: -0.3,
    };
    let cut = Cut {
        heading: Some(heading),
    };
    app.world_mut().write_message(cut);
    app.update();
    assert_eq!(*app.world().resource::<Heading>(), heading);
    assert_eq!(app.world().resource::<ViewPrefs>().zoom, 1.0);
}
