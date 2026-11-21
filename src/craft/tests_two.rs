//! Tests of what the robot of this machine owns when another player stands in the world: the bay
//! of the base mends it, and its fall leaves its own cache. The other player is drawn the way a
//! session draws it: a body with a pose and no player of this machine.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::Pack;
use super::fall::Left;
use super::stage::pack;
use crate::avatar::{Avatar, AvatarPose};
use crate::condition::{Condition, Part};
use crate::home::Base;
use crate::player::Player;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Returns a base at the centre of the test frame, on the default planet.
fn base() -> Base {
    Base {
        body: BodyId::default(),
        centre: DVec3::ZERO,
        up: DVec3::Y,
        east: DVec3::X,
        north: DVec3::NEG_Z,
        wake: DVec3::new(3.0, 0.0, 0.0),
        pad: DVec3::ZERO,
        heading: DVec3::X,
        door: DVec3::ZERO,
    }
}

/// Returns an app on the planet of the [`base`], the local robot hurt and standing at `local`, and
/// the robot of another player standing at `other`.
fn app(local: Vec3, other: Vec3) -> App {
    let anchor = Anchor {
        frame: Frame::Surface(BodyId::default()),
        origin: DVec3::ZERO,
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let mut hurt = Condition::default();
    hurt.hurt(Part::ArmL, 300.0, true);
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs(1));
    let mut app = App::new();
    app.insert_resource(anchor)
        .insert_resource(base())
        .insert_resource(hurt)
        .insert_resource(time)
        .insert_resource(pack())
        .init_resource::<Left>();
    let world = app.world_mut();
    world.spawn((Player, Transform::from_translation(local)));
    world.spawn((
        Avatar,
        AvatarPose::default(),
        Transform::from_translation(other),
    ));
    app
}

/// The bay mends the robot of this machine that stands in it, with another player in the bay too;
/// with the local robot outside and only the other player inside, it mends nothing: each machine
/// mends its own robot.
#[test]
fn the_bay_mends_the_local_robot_with_another_player_present() {
    let mut inside = app(Vec3::new(2.0, 0.0, 0.0), Vec3::new(-2.0, 0.0, 0.0));
    let before = inside.world().resource::<Condition>().of(Part::ArmL);
    let tend = super::mend::tend;
    inside
        .world_mut()
        .run_system_once(tend)
        .expect("the bay runs");
    assert!(inside.world().resource::<Condition>().of(Part::ArmL) > before);
    let mut outside = app(Vec3::new(500.0, 0.0, 0.0), Vec3::new(-2.0, 0.0, 0.0));
    outside
        .world_mut()
        .run_system_once(tend)
        .expect("the bay runs");
    assert_eq!(
        outside.world().resource::<Condition>().of(Part::ArmL),
        before
    );
}

/// A robot that is destroyed with another player in the world leaves its own cache where it fell,
/// not where the other stands, and wakes whole with an empty pack.
#[test]
fn a_fall_leaves_the_cache_of_the_local_robot_with_another_player_present() {
    let fell = Vec3::new(40.0, 0.0, 7.0);
    let mut app = app(fell, Vec3::new(-90.0, 0.0, 0.0));
    let carried = app.world().resource::<Pack>().mass();
    let wake = super::fall::wake;
    app.world_mut()
        .run_system_once(wake)
        .expect("the fall runs");
    let cache = app.world().resource::<Left>().0.clone().expect("a cache");
    assert_eq!(cache.at, [40.0, 0.0, 7.0]);
    assert!((cache.pack.mass() - carried).abs() < 1e-3);
    assert_eq!(app.world().resource::<Pack>().mass(), 0.0);
    assert!(app.world().resource::<Condition>().intact());
}
