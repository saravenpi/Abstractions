//! Tests of the air that meets a moving camera, which decides where the ambient particles appear.

use bevy::prelude::*;

use super::eye::Eye;
use super::respawn::upwind_share;

/// A camera that stands still meets the wind itself.
#[test]
fn a_still_camera_meets_the_wind() {
    let eye = Eye::default();
    let wind = Vec3::new(3.0, 0.0, -4.0);
    assert_eq!(eye.flow(wind, Vec3::ZERO), Vec2::new(3.0, -4.0));
}

/// A camera that flies into a calm sky meets air that comes from ahead, at its own speed.
#[test]
fn a_flying_camera_meets_air_from_ahead() {
    let eye = Eye::default();
    let flow = eye.flow(Vec3::ZERO, Vec3::new(0.0, -20.0, -50.0));
    assert_eq!(flow, Vec2::new(0.0, 50.0));
}

/// The local frame turns with the vertical: a camera tilted on the planet still reads its own horizontal.
#[test]
fn the_flow_follows_the_local_frame() {
    let eye = Eye {
        tilt: Quat::from_rotation_z(0.5),
        ..Eye::default()
    };
    let moving = eye.tilt * Vec3::new(10.0, 0.0, 0.0);
    let flow = eye.flow(Vec3::ZERO, moving);
    assert!((flow - Vec2::new(-10.0, 0.0)).length() < 1.0e-4);
}

/// Respawns favour the upwind face more the faster the air meets the camera, and never all of them.
#[test]
fn fast_air_favours_the_upwind_face() {
    let shares = [0.0, 5.0, 20.0, 40.0, 200.0].map(upwind_share);
    assert!(shares.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(shares[0], 0.6);
    assert!(shares[4] > 0.85 && shares[4] < 1.0);
}
