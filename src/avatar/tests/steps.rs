//! A whole life of the body, frame by frame: every joint moves at a bounded rate through every
//! transition, at any frame rate. A snap is a step that does not shrink with the frame.
//!
//! The body is driven the way the game drives it: a small simulation moves a controller state
//! and a flight gear state, and the real filler and the real dresser turn them into the pose input.

use bevy::prelude::*;

use super::life::{Act, Body, LIFE};
use super::stage::unturned;
use crate::avatar::animate::Anim;
use crate::avatar::dress::dress;
use crate::avatar::fill::{Memory, observe};
use crate::avatar::{AvatarPose, Mode};

/// Fastest any joint may move, in radians per second (meters per second for the pelvis): a
/// sprinting leg does 27, and a change of mode adds its blend.
const RATE: f32 = 34.0;

/// The largest change of a joint in one frame, and where in the life it happened.
#[derive(Debug, Clone, Copy)]
struct Worst {
    /// The change, in radians or meters.
    step: f32,
    /// The act it happened in.
    act: Act,
    /// Index of the joint in the row of a pose.
    joint: usize,
}

/// Lives the whole life at `fps` frames per second and returns the largest change of a joint
/// in one frame, with where it happened, and the modes the life went through. The first
/// frames, in which the body takes its standing pose from nothing, do not count. The whole-body
/// turn of a roll is not a joint: it follows the distance travelled, as the stride does.
fn live(fps: f32) -> (Worst, Vec<Mode>) {
    let dt = 1.0 / fps;
    let (mut body, mut memory) = (Body::default(), Memory::default());
    let (mut input, mut anim) = (AvatarPose::default(), Anim::default());
    body.motion.grounded = true;
    let (mut worst, mut seen) = (None::<Worst>, Vec::new());
    for (seconds, act) in LIFE {
        for _ in 0..(seconds * fps) as usize {
            body.act(act, dt);
            observe(&mut input, &mut memory, &body.motion, (false, dt));
            dress(&mut input, &body.gear, Vec2::ZERO, body.deep);
            input.clearance = body.height;
            let before = unturned(&anim.pose, anim.clock.turn).values();
            let pose = anim.step(&input, dt);
            let now = unturned(&pose, anim.clock.turn).values();
            let moved = before.iter().zip(now).map(|(a, b)| (a - b).abs());
            let (joint, step) = moved
                .enumerate()
                .fold((0, 0.0), |a, b| [a, b][usize::from(b.1 > a.1)]);
            let counts = anim.clock.time > 0.3 && worst.is_none_or(|worst| step > worst.step);
            worst = if counts {
                Some(Worst { step, act, joint })
            } else {
                worst
            };
            seen.extend((seen.last() != Some(&input.mode)).then_some(input.mode));
        }
    }
    (worst.expect("a life"), seen)
}

/// Through the whole life no joint ever moves faster than the bound, at 30 and at 144 frames
/// per second, and under the microscope of 1000 frames per second no step is left over: the
/// largest change in a frame shrinks with the frame, which a snap does not.
#[test]
fn no_joint_steps_through_a_whole_life() {
    for fps in [30.0, 144.0, 1000.0] {
        let (worst, seen) = live(fps);
        assert!(
            worst.step < RATE / fps,
            "at {fps} fps joint {} moves {} in a frame during {:?}",
            worst.joint,
            worst.step,
            worst.act
        );
        for mode in Mode::ALL {
            let skipped = mode == Mode::Seated;
            assert!(
                skipped || seen.contains(&mode),
                "at {fps} fps the life never passes {mode:?}: {seen:?}"
            );
        }
    }
}
