//! The crouch layer: knees and hips bent, the torso leaned forward, blended by how far down the
//! body is. Walking crouched takes shorter steps.

use super::AvatarPose;
use super::air::mix;
use super::clock::Clock;
use super::gait::{Foot, bob, foot};
use super::ground::{Stance, planted};
use super::pose::{Core, Limb, Pose};
use super::tune::{HUNCH, SHORT_STEP, SINK, SQUAT};

/// Returns `foot` with its reach and its lift cut to the short steps of a body `amount` down.
fn shortened(foot: Foot, amount: f32) -> Foot {
    let keep = 1.0 - (1.0 - SHORT_STEP) * amount;
    Foot {
        ahead: foot.ahead * keep,
        lift: foot.lift * keep,
        ..foot
    }
}

/// Returns the pose of a body fully crouched, its feet on the cycle of `clock` at the speed of
/// `input` when it `walks`, else side by side.
fn deep(input: &AvatarPose, clock: &Clock, walks: bool) -> Pose {
    let speed = input.speed;
    let mut stance = Stance {
        lift: -SINK.0 - SQUAT,
        lean: HUNCH.0,
        ..Stance::default()
    };
    if walks {
        stance.lift = -SINK.1 + 0.5 * bob(clock.phase, speed) - SQUAT;
        stance.feet = [
            shortened(foot(clock.phase, speed), 1.0),
            shortened(foot(clock.phase + 0.5, speed), 1.0),
        ];
    }
    planted(&stance)
        .with(Core::Spine, HUNCH.1)
        .with(Core::HeadPitch, -0.5 * (HUNCH.0 + HUNCH.1))
        .both(Limb::Shoulder, 0.1)
        .both(Limb::Elbow, 0.5)
}

/// Returns `base`, the pose of a body on its feet, pulled into the crouch as far as `input` says;
/// `walks` when `base` is a stride.
pub fn bent(base: Pose, input: &AvatarPose, (clock, walks): (&Clock, bool)) -> Pose {
    match input.crouch > 0.0 {
        true => mix(
            &base,
            &deep(input, clock, walks),
            input.crouch.clamp(0.0, 1.0),
        ),
        false => base,
    }
}
