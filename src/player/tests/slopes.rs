//! A 30 degree slope in a streamed world whose colliders are loaded only there: the body runs
//! down it and onto the flat ground below, and falls onto it, on the collider and never floated
//! over it by the bare ground of the planet or pushed up by it.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::{Motion, drive};
use crate::player::surface::most_lift;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::sim::{Intent, IntentOf, PlayerId};
use crate::world::Chunks;

/// Unit normal of the slope: 30 degrees, falling away ahead, along -Z.
fn normal() -> Vec3 {
    Quat::from_rotation_x(-30.0_f32.to_radians()) * Vec3::Y
}

/// Render position of a point on the top face of the slope.
const TOP: Vec3 = Vec3::new(0.0, 20.43, -0.25);

/// Height of the top face of the flat ground of the stage.
const FLAT: f32 = 10.5;

/// Returns a streamed world, so the bare ground of the planet is on wherever no collider is
/// near, with the flat ground of the stage and a 30 degree slope 60 m long over it.
fn world() -> App {
    let mut app = app(&[(PlayerId::LOCAL, 0.0)]);
    app.add_systems(Update, (gather, drive).chain())
        .init_resource::<Chunks>();
    let turn = Quat::from_rotation_x(-30.0_f32.to_radians());
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 60.0),
        Transform::from_xyz(0.0, 20.0, 0.0).with_rotation(turn),
    ));
    app
}

/// Returns how far the soles of a body centred at `at` are over the ground under its centre:
/// the slope or the flat ground, whichever is higher.
fn over(at: Vec3) -> f32 {
    let n = normal();
    let slope = TOP.y - (n.z * (at.z - TOP.z) + n.x * (at.x - TOP.x)) / n.y;
    at.y - BODY_LENGTH * 0.5 - BODY_RADIUS - slope.max(FLAT)
}

/// Puts the body at rest at render position `at`.
fn put(app: &mut App, at: Vec3) {
    let world = app.world_mut();
    let mut bodies = world.query_filtered::<(&mut Transform, &mut Motion), With<Player>>();
    for (mut transform, mut motion) in bodies.iter_mut(world) {
        transform.translation = at;
        *motion = Motion::default();
    }
}

/// Runs a frame with `intents` and returns the centre of the body and its velocity.
fn frame(app: &mut App, intents: &[Intent]) -> (Vec3, Vec3) {
    for &intent in intents {
        let player = PlayerId::LOCAL;
        app.world_mut().write_message(IntentOf { player, intent });
    }
    app.update();
    let (transform, motion, _) = body(app, PlayerId::LOCAL);
    (transform.translation, motion.velocity)
}

/// Running down the slope at the sprint and on over the flat ground at its foot, the body stays
/// on the ground under it: never more than a few centimetres over it beyond what a capsule on a
/// slope stands off, never under it, and never rising.
#[test]
fn a_run_down_a_slope_follows_it() {
    let mut app = world();
    put(&mut app, Vec3::new(0.0, 32.5, 18.0));
    for _ in 0..20 {
        frame(&mut app, &[]);
    }
    let run = [
        Intent::Move {
            forward: 1.0,
            strafe: 0.0,
        },
        Intent::Boost,
    ];
    let lean = BODY_RADIUS * (1.0 / normal().y - 1.0);
    for index in 0..120 {
        let (at, velocity) = frame(&mut app, &run);
        let gap = over(at);
        let what = format!("frame {index}: {gap} m over the ground at {at}, moving {velocity}");
        assert!(gap > -0.05 && gap < lean + 0.08, "{what}");
        assert!(velocity.y < 0.3, "{what}");
    }
    assert!(body(&mut app, PlayerId::LOCAL).1.grounded);
}

/// Falling 5 m onto the slope, the body lands on it and stays: from the first touch it never
/// rises again, and it rests with no speed upward.
#[test]
fn a_fall_onto_a_slope_lands_without_rising() {
    let mut app = world();
    put(&mut app, Vec3::new(0.0, 27.5, 4.0));
    let (mut lowest, mut touched) = (f32::MAX, false);
    for index in 0..60 {
        let (at, velocity) = frame(&mut app, &[]);
        let gap = over(at);
        touched |= gap < 0.1;
        if touched {
            let what = format!("frame {index}: at {at}, moving {velocity}, lowest {lowest}");
            assert!(at.y <= lowest + 0.02 && velocity.y < 0.1, "{what}");
        }
        assert!(gap > -0.05, "frame {index}: {gap} m under the slope");
        lowest = lowest.min(at.y);
    }
    assert!(touched, "never reached the slope");
}

/// The bare ground gives back at most what a step moved into it: the fall, the rise of the
/// steepest walkable slope over the way along it, and two centimetres; a body that only stood
/// still is never lifted more than that.
#[test]
fn the_bare_ground_never_throws_a_body_up() {
    assert!((most_lift(0.0, 0.0) - 0.02).abs() < 1e-6);
    assert!((most_lift(0.5, 0.0) - 0.52).abs() < 1e-6);
    assert!(most_lift(-1.0, 0.0) <= 0.02 + 1e-6);
    assert!(most_lift(0.0, 0.1) < 0.15 && most_lift(0.0, 0.1) > 0.12);
}
