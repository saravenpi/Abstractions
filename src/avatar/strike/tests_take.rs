//! Tests of taking a thrown javelin again: walking over one leaves it on the ground, and the
//! interact key takes it only once the robot looks at it and the prompt named the take.

use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::*;
use bevy_persistent::StorageFormat;

use super::super::Avatar;
use super::flight::{Rest, Spear};
use super::held::Held;
use super::pull::pull;
use super::stage::spear;
use crate::player::{Interact, Interacting, Player, Verb};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};
use crate::sim::PlayerId;

/// Where the eyes of the robot are, in the world of the test.
const EYE: Vec3 = Vec3::new(0.0, 0.7, -0.16);

/// Returns default settings in a file of their own, so that tests running at once never share one.
fn settings() -> Persistent<Settings> {
    static MADE: AtomicUsize = AtomicUsize::new(0);
    let made = MADE.fetch_add(1, Ordering::Relaxed);
    let name = format!("abstractions-take-{}-{made}", std::process::id());
    Persistent::<Settings>::builder()
        .name("settings")
        .format(StorageFormat::Toml)
        .path(std::env::temp_dir().join(name).join("settings.toml"))
        .default(Settings::default())
        .build()
        .expect("settings in a scratch directory")
}

/// Returns an app with the take, the local robot standing at the origin with empty hands, its
/// camera at the eyes looking level ahead, and a javelin lying across its feet. Returns the app,
/// the body, the camera and the javelin.
fn stage() -> (App, Entity, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(Interacting)
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(settings())
        .add_systems(Update, pull);
    let world = app.world_mut();
    let player = world
        .spawn((Player, PlayerId::LOCAL, Transform::IDENTITY))
        .id();
    let body = world.spawn((Avatar, ChildOf(player))).id();
    let camera = world
        .spawn((GameCamera, GlobalTransform::from_translation(EYE)))
        .id();
    let lying = Transform::from_xyz(0.6, -0.86, 0.0)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::X));
    let javelin = Spear {
        item: spear(),
        velocity: Vec3::ZERO,
        rest: Rest::Lying,
        thrower: player,
        body,
        was: lying.translation,
        lent: None,
    };
    let javelin = world.spawn((javelin, GlobalTransform::from(lying))).id();
    (app, body, camera, javelin)
}

/// Returns the action the prompt names this frame and what it names.
fn named(app: &App) -> Option<(Verb, &'static str)> {
    let chosen = app.world().resource::<Interact>().chosen();
    chosen.map(|offer| (offer.verb, offer.thing))
}

/// Standing over a lying javelin and looking ahead leaves it on the ground and offers nothing;
/// looking down at it the prompt names the take, and the interact key then puts it in the hands.
#[test]
fn a_javelin_is_taken_with_the_key_only() {
    let (mut app, body, camera, javelin) = stage();
    for _ in 0..3 {
        app.update();
    }
    let stays = app.world().get_entity(javelin).is_ok();
    assert!(stays && named(&app).is_none(), "walking over it took it");
    let down = Transform::from_translation(EYE).looking_at(Vec3::new(0.0, -0.86, 0.0), Vec3::Y);
    app.world_mut()
        .entity_mut(camera)
        .insert(GlobalTransform::from(down));
    app.update();
    app.update();
    assert_eq!(named(&app), Some((Verb::Take, "javelin")));
    assert!(app.world().get_entity(javelin).is_ok(), "looking took it");
    let settings = app.world().resource::<Persistent<Settings>>();
    let key = settings.controls.keys.ship.board;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
    app.update();
    let taken = app.world().get_entity(javelin).is_err();
    assert!(
        taken && app.world().get::<Held>(body).is_some(),
        "the key did not take it"
    );
}
