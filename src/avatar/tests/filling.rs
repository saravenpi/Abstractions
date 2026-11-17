//! Filling the pose input from the player controller's movement state.

use bevy::prelude::*;

use super::FRAME;
use crate::avatar::fill::{LAND_SECONDS, Memory, classify, hardness, observe};
use crate::avatar::{AvatarPose, Mode};
use crate::player::Motion;

/// Returns a movement state with the given velocity, on the ground or not.
fn moving(velocity: Vec3, grounded: bool) -> Motion {
    Motion {
        velocity,
        grounded,
        ..default()
    }
}

/// The movement state names the mode: still, walking, running, rising, falling, swimming, seated.
#[test]
fn movement_names_the_mode() {
    assert_eq!(classify(&moving(Vec3::ZERO, true), false), Mode::Idle);
    assert_eq!(
        classify(&moving(Vec3::new(3.0, 0.0, 4.0), true), false),
        Mode::Walk
    );
    assert_eq!(classify(&moving(Vec3::Z * 8.5, true), false), Mode::Run);
    assert_eq!(
        classify(&moving(Vec3::new(2.0, 9.0, 0.0), false), false),
        Mode::Jump
    );
    assert_eq!(
        classify(&moving(Vec3::new(2.0, -9.0, 0.0), false), false),
        Mode::Fall
    );
    let swimming = Motion {
        swimming: true,
        ..moving(Vec3::X, false)
    };
    assert_eq!(classify(&swimming, false), Mode::Swim);
    assert_eq!(classify(&swimming, true), Mode::Seated);
}

/// Falls at `fall` meters per second, touches down, and returns the pose once the landing is
/// over with the number of frames it lasted.
fn touch_down(fall: f32) -> (AvatarPose, usize) {
    let (mut pose, mut memory) = (AvatarPose::default(), Memory::default());
    observe(
        &mut pose,
        &mut memory,
        &moving(Vec3::NEG_Y * fall, false),
        (false, FRAME),
    );
    assert_eq!((pose.mode, pose.vertical_speed), (Mode::Fall, -fall));
    for frames in 0..1000 {
        observe(
            &mut pose,
            &mut memory,
            &moving(Vec3::ZERO, true),
            (false, FRAME),
        );
        if pose.mode != Mode::Land {
            return (pose, frames);
        }
    }
    panic!("the landing never ends");
}

/// Touching down after a fall lands for a moment, as hard as the fall was fast.
#[test]
fn touching_down_lands() {
    for (fall, hard) in [(4.0, false), (40.0, true)] {
        let (pose, frames) = touch_down(fall);
        assert_eq!(pose.mode, Mode::Idle);
        assert!(
            (frames as f32 * FRAME - LAND_SECONDS).abs() <= FRAME,
            "landed for {frames} frames"
        );
        assert_eq!(
            pose.landed_hard > 0.5,
            hard,
            "fall at {fall} m/s is {}",
            pose.landed_hard
        );
    }
    assert_eq!((hardness(0.0), hardness(100.0)), (0.0, 1.0));
}

/// The walked distance grows with the ground speed only, so the legs stop in the air and in water.
#[test]
fn distance_grows_on_the_ground_only() {
    let (mut pose, mut memory) = (AvatarPose::default(), Memory::default());
    for _ in 0..60 {
        observe(
            &mut pose,
            &mut memory,
            &moving(Vec3::X * 5.0, true),
            (false, FRAME),
        );
    }
    assert!(
        (pose.stride_distance - 5.0).abs() < 1e-3,
        "walked {}",
        pose.stride_distance
    );
    assert_eq!((pose.mode, pose.speed), (Mode::Walk, 5.0));
    let swimming = Motion {
        swimming: true,
        ..moving(Vec3::X * 3.0, false)
    };
    for motion in [moving(Vec3::new(5.0, -3.0, 0.0), false), swimming] {
        let before = pose.stride_distance;
        observe(&mut pose, &mut memory, &motion, (false, FRAME));
        assert_eq!(pose.stride_distance, before);
    }
    assert!((pose.airspeed - 3.0).abs() < 1e-5);
}

/// A sprint through a touch-down keeps running: the legs are not frozen in a landing pose.
#[test]
fn a_running_landing_keeps_running() {
    let (mut pose, mut memory) = (AvatarPose::default(), Memory::default());
    observe(
        &mut pose,
        &mut memory,
        &moving(Vec3::new(8.5, -6.0, 0.0), false),
        (false, FRAME),
    );
    observe(
        &mut pose,
        &mut memory,
        &moving(Vec3::X * 8.5, true),
        (false, FRAME),
    );
    assert_eq!(pose.mode, Mode::Run);
}
