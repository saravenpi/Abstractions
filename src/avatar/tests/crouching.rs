//! The crouch layer: the pelvis sinks, knees and trunk bend, the feet stay on the ground, the joints
//! stay in their limits and a crouched stride is shorter.

use bevy::prelude::*;

use crate::avatar::AvatarPose;
use crate::avatar::Mode;
use crate::avatar::blend::pose_of;
use crate::avatar::clock::Clock;
use crate::avatar::pose::{Core, Limb};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::{ANKLE_RISE, SOLE_Y, SQUAT};

/// Returns the state of a body in `mode` going `speed`, `crouch` down.
fn state(mode: Mode, speed: f32, crouch: f32) -> AvatarPose {
    AvatarPose {
        mode,
        speed,
        crouch,
        ..default()
    }
}

/// The pelvis sinks by the squat, the knees and the torso bend, and both soles stay on the ground.
#[test]
fn a_crouched_body_bends_and_keeps_its_feet_down() {
    let clock = Clock::default();
    let up = pose_of(Mode::Idle, &state(Mode::Idle, 0.0, 0.0), &clock);
    let down = pose_of(Mode::Idle, &state(Mode::Idle, 0.0, 1.0), &clock);
    let sunk = up.core(Core::Lift) - down.core(Core::Lift);
    assert!((sunk - SQUAT).abs() < 0.02, "sank by {sunk}");
    assert!(down.core(Core::Spine) > up.core(Core::Spine) + 0.05);
    assert!(down.core(Core::Pitch) > up.core(Core::Pitch) + 0.1);
    assert!(down.limb(Limb::Knee, 0) > up.limb(Limb::Knee, 0) + 0.5);
    for boot in [Seg::BootL, Seg::BootR] {
        let ankle = globals(&down)[boot as usize].translation.y;
        assert!(
            (ankle - (SOLE_Y + ANKLE_RISE)).abs() < 0.05,
            "ankle at {ankle}"
        );
    }
}

/// Half way down is half way between the two poses.
#[test]
fn the_crouch_is_blended_by_its_amount() {
    let clock = Clock::default();
    let at = |crouch: f32| pose_of(Mode::Idle, &state(Mode::Idle, 0.0, crouch), &clock);
    let (up, half, down) = (at(0.0), at(0.5), at(1.0));
    let middle = 0.5 * (up.core(Core::Lift) + down.core(Core::Lift));
    assert!((half.core(Core::Lift) - middle).abs() < 1e-4);
}

/// A crouched stride swings its feet less far than a standing one at the same speed.
#[test]
fn a_crouched_stride_takes_shorter_steps() {
    let clock = Clock {
        phase: 0.7,
        ..Clock::default()
    };
    let reach = |crouch: f32| {
        let pose = pose_of(Mode::Walk, &state(Mode::Walk, 2.0, crouch), &clock);
        (pose.limb(Limb::Hip, 0) - pose.limb(Limb::Hip, 1)).abs()
    };
    assert!(
        reach(1.0) < reach(0.0),
        "{} against {}",
        reach(1.0),
        reach(0.0)
    );
}
