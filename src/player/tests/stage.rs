//! A small world for the controller alone: flat ground, player bodies, the physics, no window.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use avian3d::prelude::*;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_persistent::StorageFormat;

use super::tangent_anchor;
use crate::gear::Gear;
use crate::player::controller::Motion;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::settings::{Persistent, Settings};
use crate::sim::{IntentOf, PlayerId};
use crate::universe::STANDARD_GRAVITY;
use crate::view::{Heading, Look, Target, ViewPrefs};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Returns default settings in a file of their own, so that tests running at once never share one.
pub(super) fn settings() -> Persistent<Settings> {
    static MADE: AtomicUsize = AtomicUsize::new(0);
    let made = MADE.fetch_add(1, Ordering::Relaxed);
    let name = format!("abstractions-intents-{}-{made}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(name).join("settings.toml"))
        .default(Settings::default())
        .build()
        .expect("settings in a scratch directory")
}

/// Spawns a flat ground 10 m above the sea and, for each pair of `bodies`, a body of that
/// player standing on it at render `x`.
fn stand(world: &mut World, bodies: &[(PlayerId, f32)]) {
    let ground = Transform::from_xyz(0.0, 10.0, 0.0);
    world.spawn((
        RigidBody::Static,
        Collider::cuboid(400.0, 1.0, 400.0),
        ground,
    ));
    let height = 10.5 + BODY_LENGTH * 0.5 + BODY_RADIUS + 0.05;
    for &(id, x) in bodies {
        world.spawn((
            (Player, id),
            (Motion::default(), Gear::default()),
            (RigidBody::Kinematic, CustomPositionIntegration),
            Collider::capsule(BODY_RADIUS, BODY_LENGTH),
            Transform::from_xyz(x, height, 0.0),
        ));
    }
}

/// Returns a world on a 3 km planet with the bodies of [`stand`], stepped at 30 frames a second.
/// Its first frame, which lasts no time, is already run.
pub(super) fn app(bodies: &[(PlayerId, f32)]) -> App {
    let mut app = App::new();
    let globe = Arc::new(Globe::new(4, 3000.0, STANDARD_GRAVITY));
    let frame = TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 30.0));
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .add_plugins(PhysicsPlugins::default())
        .init_asset::<Mesh>()
        .insert_resource(frame)
        .insert_resource(settings())
        .insert_resource(tangent_anchor(DVec3::Y))
        .insert_resource(Terrain::on(globe.clone(), globe.face_chart(Face::ALL[0])))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<Heading>()
        .init_resource::<Look>()
        .init_resource::<ViewPrefs>()
        .init_resource::<Target>()
        .init_resource::<crate::player::eyes::WingView>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<IntentOf>();
    stand(app.world_mut(), bodies);
    app.finish();
    app.cleanup();
    app.update();
    app
}

/// Returns the pose, motion and gear of the body of `id`.
pub(super) fn body(app: &mut App, id: PlayerId) -> (Transform, Motion, Gear) {
    let world = app.world_mut();
    let mut bodies =
        world.query_filtered::<(&PlayerId, &Transform, &Motion, &Gear), With<Player>>();
    let found = bodies.iter(world).find(|body| *body.0 == id);
    let (_, transform, motion, gear) = found.expect("the body");
    (*transform, *motion, *gear)
}
