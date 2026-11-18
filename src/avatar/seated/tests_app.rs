//! The seat in a running game: the seated body is drawn, has no collider, holds the controls
//! of its ship, and is given back to the animation on leaving.

use std::time::Duration;

use avian3d::prelude::{Collider, ColliderDisabled};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::avatar::animate::{Anim, Bone};
use crate::avatar::rig::{Seg, locals};
use crate::avatar::{Avatar, Shown};
use crate::player::{Motion, Player, seat};
use crate::ship::SEAT;
use crate::ship::cockpit::{Cockpit, Levers};
use crate::space::{Location, Travel};
use crate::universe::BodyId;
use crate::view::{BodyFade, Heading};

/// Returns an app with the avatar systems and a player seated in the ship, seen from behind.
fn aboard() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Heading>()
        .insert_resource(BodyFade(1.0))
        .insert_resource(Location {
            body: BodyId::default(),
            travel: Travel::InShip,
        })
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
    crate::avatar::systems(&mut app);
    let player = crew(&mut app);
    (0..3).for_each(|_| app.update());
    (app, player)
}

/// Spawns a player and the ship it pilots, the stick held back and right, and seats the player.
fn crew(app: &mut App) -> Entity {
    let body = (Player, Motion::default(), Collider::capsule(0.3, 1.2));
    let at = Transform::from_xyz(5.0, 2.0, -3.0).with_rotation(Quat::from_rotation_y(0.7));
    let world = app.world_mut();
    let player = world.spawn((body, at, Visibility::default())).id();
    let shown = Levers {
        stick: Vec3::new(0.5, 0.0, -1.0),
        throttle: 0.7,
    };
    world.spawn(Cockpit {
        shown,
        pilot: Some(player),
        ..default()
    });
    seat(world, true);
    player
}

/// Returns the visibility of the root of the body and of its head.
fn drawn(app: &mut App) -> (Visibility, Visibility) {
    let mut roots = app
        .world_mut()
        .query_filtered::<&Visibility, With<Avatar>>();
    let root = *roots.single(app.world()).expect("one body");
    let mut bones = app.world_mut().query::<(&Bone, &Visibility)>();
    let head = bones
        .iter(app.world())
        .find(|(bone, _)| bone.0 == Seg::Head);
    (root, *head.expect("a head").1)
}

/// Returns where the left and the right palm are in the frame of the `player` entity.
fn palms(app: &mut App, player: Entity) -> [Vec3; 2] {
    let mut hands = app.world_mut().query::<(&Bone, &GlobalTransform)>();
    let body = *app.world().get::<GlobalTransform>(player).expect("a place");
    let back = body.affine().inverse();
    [Seg::HandL, Seg::HandR].map(|seg| {
        let found = hands.iter(app.world()).find(|(bone, _)| bone.0 == seg);
        let hand = found.expect("a hand").1;
        back.transform_point3(hand.transform_point(Vec3::NEG_Y * 0.045))
    })
}

/// The seated body is drawn, whole from outside and without its head at the eyes, it sits in
/// the seat with its palms on the grips at once, and its collider is off while it sits.
#[test]
fn the_seated_body_is_drawn_and_has_no_collider() {
    let (mut app, player) = aboard();
    assert!(app.world().get::<ColliderDisabled>(player).is_some());
    assert_eq!(*app.world().resource::<Shown>(), Shown::Full);
    assert_eq!(
        drawn(&mut app),
        (Visibility::Inherited, Visibility::Inherited)
    );
    for (side, palm) in palms(&mut app, player).into_iter().enumerate() {
        let mut ships = app.world_mut().query::<&Cockpit>();
        let levers = ships.single(app.world()).expect("a ship").shown;
        let grip = levers.grips()[side] - SEAT.as_vec3();
        assert!(palm.distance(grip) < 0.015, "hand {side} at {palm}");
    }
    app.insert_resource(BodyFade(0.0));
    app.update();
    assert_eq!(*app.world().resource::<Shown>(), Shown::FirstPerson);
    assert_eq!(drawn(&mut app), (Visibility::Inherited, Visibility::Hidden));
}

/// Leaving the ship puts the root of the body back on the player at once, switches the
/// collider on, and lets the animation take the pose back without a jump.
#[test]
fn leaving_the_seat_gives_the_body_back() {
    let (mut app, player) = aboard();
    (0..60).for_each(|_| app.update());
    app.world_mut().resource_mut::<Location>().travel = Travel::OnFoot;
    let mut ships = app.world_mut().query::<&mut Cockpit>();
    ships.single_mut(app.world_mut()).expect("a ship").pilot = None;
    seat(app.world_mut(), false);
    let mut roots = app
        .world_mut()
        .query_filtered::<(&Transform, &Anim), With<Avatar>>();
    let mut bones = app.world_mut().query::<(&Bone, &Transform)>();
    let thigh = |app: &App, bones: &mut QueryState<(&Bone, &Transform)>| {
        let found = bones
            .iter(app.world())
            .find(|(bone, _)| bone.0 == Seg::ThighR);
        found.expect("a thigh").1.rotation
    };
    let mut last = thigh(&app, &mut bones);
    for _ in 0..60 {
        app.update();
        let now = thigh(&app, &mut bones);
        assert!(now.angle_between(last) < 0.12, "the legs jumped on leaving");
        last = now;
    }
    assert!(app.world().get::<ColliderDisabled>(player).is_none());
    let (root, anim) = roots.single(app.world()).expect("one body");
    assert_eq!(root.translation, Vec3::ZERO);
    let frames = locals(&anim.pose);
    assert!(last.angle_between(frames[Seg::ThighR as usize].rotation) < 1.0e-4);
}
