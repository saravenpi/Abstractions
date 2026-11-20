//! Changing mode never pops: the pose moves by a small bounded step per frame, from any mode to any other.

use bevy::prelude::*;

use super::{FRAME, gap, scene};
use crate::avatar::animate::Anim;
use crate::avatar::blend::{MODES, Weights};
use crate::avatar::pose::{Core, Limb};
use crate::avatar::tune::{BLEND_SECONDS, CROUCH};
use crate::avatar::{AvatarPose, Mode};

/// Largest change of any joint in one frame at sixty frames per second that a change of mode may
/// cause, in radians (or meters for the two offsets of the pelvis).
const BLEND_STEP: f32 = 0.16;

/// Largest change of any joint in one frame that a mode may make by itself, flat out at a sprint.
const MOTION_STEP: f32 = 0.45;

/// Returns the state of `mode` in a scene that does not move by itself: only the mode changes.
fn still(mode: Mode) -> AvatarPose {
    AvatarPose {
        speed: 0.0,
        stride_distance: 0.0,
        ..scene(mode, 0.35)
    }
}

/// Runs `frames` frames of `input` and returns the largest change of a joint in one frame.
fn run(anim: &mut Anim, input: &mut AvatarPose, frames: usize) -> f32 {
    let mut worst = 0.0_f32;
    for _ in 0..frames {
        input.stride_distance += input.speed * FRAME;
        let before = anim.pose;
        worst = worst.max(gap(&before, &anim.step(input, FRAME)));
    }
    worst
}

/// Returns an animation settled in `input`.
fn settled(input: &mut AvatarPose) -> Anim {
    let mut anim = Anim {
        weights: Weights::only(input.mode),
        ..default()
    };
    run(&mut anim, input, 90);
    anim
}

/// From any mode to any other, the pose never jumps: no joint moves more than the bound in a frame.
#[test]
fn any_change_of_mode_is_continuous() {
    for (from, to) in Mode::ALL
        .into_iter()
        .flat_map(|a| Mode::ALL.map(|b| (a, b)))
    {
        let mut anim = settled(&mut still(from));
        let step = run(&mut anim, &mut still(to), 60);
        assert!(step < BLEND_STEP, "{from:?} to {to:?} jumps by {step}");
        assert!(
            anim.weights.0[to as usize] == 1.0,
            "{from:?} to {to:?} never settles"
        );
    }
}

/// A change of mode interrupted by another one, at any moment, does not jump either.
#[test]
fn interrupted_changes_are_continuous() {
    let chains = [
        [Mode::Fly, Mode::Fall, Mode::Land],
        [Mode::Glide, Mode::Walk, Mode::Idle],
        [Mode::Jump, Mode::BootFall, Mode::Glide],
        [Mode::Swim, Mode::Dive, Mode::Seated],
        [Mode::Run, Mode::Jump, Mode::Fly],
    ];
    for chain in chains {
        for cut in [1, 4, 9, 14] {
            let mut anim = settled(&mut still(chain[0]));
            let first = run(&mut anim, &mut still(chain[1]), cut);
            let second = run(&mut anim, &mut still(chain[2]), 60);
            assert!(
                first.max(second) < BLEND_STEP,
                "{chain:?} cut at {cut}: {first} {second}"
            );
        }
    }
}

/// At full speed too: leaving flight for a fall, and a glide for a walk, add only the blend step
/// to the motion the modes make by themselves.
#[test]
fn changes_at_speed_stay_bounded() {
    for (from, to) in [
        (Mode::Fly, Mode::Fall),
        (Mode::Glide, Mode::Walk),
        (Mode::Run, Mode::Jump),
    ] {
        let mut input = scene(from, 1.0);
        let mut anim = settled(&mut input);
        input.mode = to;
        let step = run(&mut anim, &mut input, 60);
        assert!(
            step < BLEND_STEP + MOTION_STEP,
            "{from:?} to {to:?} at speed jumps by {step}"
        );
    }
    for mode in Mode::ALL {
        let mut input = scene(mode, 1.0);
        let mut anim = settled(&mut input);
        let step = run(&mut anim, &mut input, 240);
        assert!(step < MOTION_STEP, "{mode:?} moves by {step} in a frame");
    }
}

/// The weights sum to one once eased, and a change is complete after the blend time: the short
/// one between poses on the feet, the long one when the body turns over.
#[test]
fn weights_blend_in_the_documented_time() {
    let changes = [
        (Mode::Idle, Mode::Run, BLEND_SECONDS.0),
        (Mode::Glide, Mode::Walk, BLEND_SECONDS.1),
    ];
    for (from, to, seconds) in changes {
        let mut weights = Weights::only(from);
        let frames = (seconds / FRAME).ceil() as usize;
        for frame in 0..=frames {
            let eased = weights.eased();
            assert!((eased.iter().sum::<f32>() - 1.0).abs() < 1e-5);
            assert!(eased.iter().all(|w| (0.0..=1.0).contains(w)));
            assert!(
                frame == frames || weights.0[to as usize] < 1.0,
                "{from:?} to {to:?} is early"
            );
            weights.step(to, FRAME, false);
        }
        assert_eq!(weights, Weights::only(to), "{from:?} to {to:?} is late");
    }
    assert_eq!(Weights::default().0.len(), MODES);
}

/// A landing bends the knees, deeper when harder, without a jump, and the body stands up again.
#[test]
fn landing_bends_the_knees_and_recovers() {
    let mut depths = [0.0_f32; 2];
    for (slot, hard) in [0.0, 1.0].into_iter().enumerate() {
        let mut input = AvatarPose {
            mode: Mode::Fall,
            ..default()
        };
        let mut anim = settled(&mut input);
        let rest = settled(&mut AvatarPose::default()).pose;
        input = AvatarPose {
            mode: Mode::Land,
            landed_hard: hard,
            ..default()
        };
        let mut step = run(&mut anim, &mut input, 10);
        depths[slot] = anim.clock.crouch;
        assert!(anim.pose.core(Core::Lift) < rest.core(Core::Lift) - 0.01);
        input.mode = Mode::Idle;
        step = step.max(run(&mut anim, &mut input, 20));
        assert!(step < BLEND_STEP, "landing {hard} jumps by {step}");
        run(&mut anim, &mut input, 90);
        assert!(anim.clock.crouch.abs() < 2e-3, "still crouched");
        assert!((anim.pose.limb(Limb::Knee, 0) - rest.limb(Limb::Knee, 0)).abs() < 0.05);
    }
    assert!(
        depths[0] > 0.02 && depths[1] > depths[0] + 0.05 && depths[1] <= CROUCH.1,
        "{depths:?}"
    );
}
