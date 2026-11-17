//! Every mode gives a sane pose: finite, inside the joint limits, and shaped as its name says.

use bevy::prelude::*;

use super::{samples, scene};
use crate::avatar::blend::{Weights, blended, pose_of};
use crate::avatar::clock::Clock;
use crate::avatar::pose::{CORE_LIMITS, CORES, LIMB_LIMITS, LIMBS, Limb, Pose};
use crate::avatar::rig::{Seg, globals, locals};
use crate::avatar::{AvatarPose, Mode};

/// Asserts that every joint of `pose` is finite and inside its documented limits.
fn assert_inside(pose: &Pose, what: &str) {
    for (index, value) in pose.values().into_iter().enumerate() {
        let (low, high) = match index.checked_sub(CORES) {
            None => CORE_LIMITS[index],
            Some(limb) => LIMB_LIMITS[limb % LIMBS],
        };
        assert!(value.is_finite(), "{what}: joint {index} is {value}");
        assert!(
            (low..=high).contains(&value),
            "{what}: joint {index} = {value} outside {low}..{high}"
        );
    }
}

/// Returns the pose of `mode` flying with the given bank and path pitch, looking ahead.
fn flying(mode: Mode, bank: f32, pitch: f32) -> Pose {
    let input = AvatarPose {
        bank,
        pitch,
        look: Vec2::ZERO,
        ..scene(mode, 0.5)
    };
    pose_of(mode, &input, &Clock::default())
}

/// Every mode, at every sampled state and instant, stays finite and inside the joint limits.
#[test]
fn every_mode_stays_inside_the_joint_limits() {
    for mode in Mode::ALL {
        for (input, clock) in samples(mode) {
            let pose = pose_of(mode, &input, &clock);
            assert_inside(&pose, &format!("{mode:?}"));
            assert_inside(
                &blended(&Weights::only(mode), &input, &clock),
                &format!("{mode:?} looking"),
            );
            for frame in locals(&pose) {
                assert!(frame.is_finite(), "{mode:?}: {frame:?}");
            }
        }
    }
}

/// Knees and elbows never bend backwards, in any mode and halfway between any two modes.
#[test]
fn knees_and_elbows_never_bend_backwards() {
    let hinges = [Limb::Knee, Limb::Elbow]
        .into_iter()
        .flat_map(|joint| [(joint, 0), (joint, 1)]);
    for (a, b) in Mode::ALL
        .into_iter()
        .flat_map(|a| Mode::ALL.map(|b| (a, b)))
    {
        for (input, clock) in samples(a) {
            let (first, second) = (pose_of(a, &input, &clock), pose_of(b, &input, &clock));
            let pose = Pose::default().plus(&first, 0.5).plus(&second, 0.5);
            for (joint, side) in hinges.clone() {
                let bend = pose.limb(joint, side);
                assert!(bend >= 0.0, "{a:?}+{b:?}: {joint:?} bent by {bend}");
            }
            assert_inside(&pose, &format!("{a:?}+{b:?}"));
        }
    }
}

/// The flight poses are prone: head ahead of the hips, boots behind, and the glide spreads wide.
#[test]
fn flight_poses_are_prone() {
    for mode in [Mode::Glide, Mode::Fly] {
        let at = globals(&flying(mode, 0.0, 0.0)).map(|frame| frame.translation);
        let (head, hips, boot) = (
            at[Seg::Head as usize],
            at[Seg::Pelvis as usize],
            at[Seg::BootL as usize],
        );
        assert!(head.z < hips.z - 0.4, "{mode:?}: head {head}");
        assert!(boot.z > hips.z + 0.5, "{mode:?}: boot {boot}");
        assert!(
            (head.y - boot.y).abs() < 0.35,
            "{mode:?}: not level, {head} {boot}"
        );
    }
    let span = |mode: Mode| {
        let at = globals(&flying(mode, 0.0, 0.0)).map(|frame| frame.translation);
        at[Seg::HandR as usize].x - at[Seg::HandL as usize].x
    };
    assert!(span(Mode::Glide) > 1.1, "glide span {}", span(Mode::Glide));
    assert!(span(Mode::Fly) < 0.75, "fly span {}", span(Mode::Fly));
}

/// Banking drops the inner hand, and the body pitches with the flight path.
#[test]
fn flight_banks_and_pitches() {
    for mode in [Mode::Glide, Mode::Fly] {
        let hands = |bank: f32| {
            let at = globals(&flying(mode, bank, 0.0)).map(|frame| frame.translation);
            at[Seg::HandR as usize].y - at[Seg::HandL as usize].y
        };
        assert!(
            hands(0.6) < hands(0.0) - 0.1,
            "{mode:?}: right bank {}",
            hands(0.6)
        );
        assert!(
            hands(-0.6) > hands(0.0) + 0.1,
            "{mode:?}: left bank {}",
            hands(-0.6)
        );
        let nose = |pitch: f32| {
            let at = globals(&flying(mode, 0.0, pitch)).map(|frame| frame.translation);
            at[Seg::Head as usize].y - at[Seg::Pelvis as usize].y
        };
        assert!(
            nose(0.5) > nose(0.0) + 0.15,
            "{mode:?}: climb {}",
            nose(0.5)
        );
        assert!(
            nose(-0.5) < nose(0.0) - 0.15,
            "{mode:?}: descent {}",
            nose(-0.5)
        );
    }
}

/// The boots point down while falling on them and back in powered flight, where they push.
#[test]
fn boots_point_where_they_push() {
    let soles = |mode: Mode| {
        let frames = globals(&flying(mode, 0.0, 0.0));
        [Seg::BootL, Seg::BootR].map(|boot| frames[boot as usize].rotation * Vec3::NEG_Y)
    };
    for sole in soles(Mode::BootFall) {
        assert!(sole.dot(Vec3::NEG_Y) > 0.99, "boot fall sole {sole}");
    }
    for sole in soles(Mode::Fly) {
        assert!(sole.dot(Vec3::Z) > 0.95, "flight sole {sole}");
    }
    let upright = globals(&flying(Mode::BootFall, 0.0, 0.0));
    assert!(upright[Seg::Head as usize].translation.y > 0.4);
    assert!(flying(Mode::BootFall, 0.0, 0.0).limb(Limb::Knee, 0) > 0.3);
}

/// The swimmer lies level when moving and the seated body folds at the hips and the knees.
#[test]
fn swimming_is_level_and_sitting_is_folded() {
    let input = AvatarPose {
        speed: 3.0,
        ..scene(Mode::Swim, 0.0)
    };
    let at =
        globals(&pose_of(Mode::Swim, &input, &Clock::default())).map(|frame| frame.translation);
    assert!(at[Seg::Head as usize].z < at[Seg::Pelvis as usize].z - 0.4);
    let seat = pose_of(Mode::Seated, &scene(Mode::Seated, 0.0), &Clock::default());
    assert!(seat.limb(Limb::Hip, 0) > 1.2 && seat.limb(Limb::Knee, 1) > 1.2);
    let frames = globals(&seat);
    assert!(
        frames[Seg::ShinL as usize].translation.z < -0.3,
        "knees ahead when seated"
    );
}
