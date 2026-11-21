//! Tests of the throw: a javelin leaves from the hand on the line through the point the crosshair
//! picked, whatever way the body faces and however the place it stands on tilts, with no loft
//! added to the aim.

use bevy::prelude::*;

use super::javelin::launch;

/// Returns how far the line from `from` along the unit `way` passes from `point`.
fn miss(from: Vec3, way: Vec3, point: Vec3) -> f32 {
    let along = (point - from).dot(way);
    (point - from - way * along).length()
}

/// The javelin starts with its tip on the line from the hand to the aimed point and leaves along it,
/// in the frame of the body and in the world, for bodies turned and tilted every way.
#[test]
fn a_javelin_leaves_through_the_aimed_point() {
    let grip = Vec3::new(0.43, 0.42, 0.5);
    let aims = [
        Vec3::new(0.0, 0.7, -30.0),
        Vec3::new(-6.0, -0.9, -4.0),
        Vec3::new(3.0, 9.0, -12.0),
    ];
    let up = Vec3::new(0.3, 0.8, -0.5).normalize();
    for at in aims {
        for turn in [0.0_f32, 0.9, -2.4] {
            let player = Transform::from_xyz(5.0, 2.0, -7.0)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, up) * Quat::from_rotation_y(turn));
            let (tip, way) = launch(grip, at, 1.1);
            assert!((tip.distance(grip) - 1.1).abs() < 1e-4);
            let (tip, way) = (player.transform_point(tip), player.rotation * way);
            let gap = miss(tip, way, player.transform_point(at));
            assert!(gap < 1e-3, "aim {at}, turn {turn}: {gap} m off");
        }
    }
}

/// A point straight ahead at the height of the hand is thrown at level: nothing lofts the aim.
#[test]
fn a_level_aim_throws_level() {
    let grip = Vec3::new(0.4, 0.4, 0.3);
    let (_, way) = launch(grip, grip + Vec3::NEG_Z * 40.0, 1.2);
    assert!(way.distance(Vec3::NEG_Z) < 1e-5, "{way}");
}
