//! The ground a body is put on: the top of a fixed collider, found from inside a slope too,
//! never a moving body such as the ship, and nothing where no collider is loaded.

use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::stage::app;
use crate::player::Floor;
use crate::sim::PlayerId;

/// Returns the ground [`Floor::under`] finds under render position `at` of `app`.
fn under(app: &mut App, at: Vec3) -> Option<Vec3> {
    let found = app
        .world_mut()
        .run_system_once(move |floor: Floor| floor.under(at, Vec3::Y));
    found.expect("the cast runs")
}

/// A point 1 m inside a 30 degree slope finds the top of the slope above it, a kinematic box
/// over the flat ground is passed through down to the ground, and past the edge of the ground
/// there is nothing.
#[test]
fn the_floor_is_the_top_of_a_fixed_collider() {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    let tilt = Quat::from_rotation_z(30_f32.to_radians());
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 2.0, 20.0),
        Transform::from_xyz(100.0, 20.0, 0.0).with_rotation(tilt),
    ));
    app.world_mut().spawn((
        RigidBody::Kinematic,
        Collider::cuboid(6.0, 3.0, 6.0),
        Transform::from_xyz(-100.0, 15.0, 0.0),
    ));
    for _ in 0..3 {
        app.update();
    }
    let top = 20.0 + 1.0 / 30_f32.to_radians().cos();
    let slope = under(&mut app, Vec3::new(100.0, top - 1.0, 0.0)).expect("the slope");
    assert!((slope.y - top).abs() < 0.01, "{slope}");
    let ground = under(&mut app, Vec3::new(-100.0, 11.0, 0.0)).expect("the ground");
    assert!((ground.y - 10.5).abs() < 0.01, "{ground}");
    assert_eq!(under(&mut app, Vec3::new(1000.0, 11.0, 0.0)), None);
}
