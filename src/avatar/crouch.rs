//! The crouch layer: hips down and back over the heels, knees bent, the torso leaned forward a
//! little for balance and the head held level against that lean, blended by how far down the body
//! is. Walking crouched takes shorter steps and keeps the trunk still, so the head stays steady.

use super::AvatarPose;
use super::air::mix;
use super::clock::Clock;
use super::gait::{Foot, bob, foot};
use super::ground::{Stance, planted};
use super::pose::{Core, Limb, Pose};
use super::tune::{HEELS, HUNCH, SHORT_STEP, SINK, SQUAT};

/// Returns `foot` with its reach and its lift cut to the short steps of a fully crouched body,
/// and set [`HEELS`] behind the hips.
fn shortened(foot: Foot) -> Foot {
    Foot {
        ahead: foot.ahead * SHORT_STEP - HEELS,
        lift: foot.lift * SHORT_STEP,
        ..foot
    }
}

/// Returns the pose of a body fully crouched, its feet on the cycle of `clock` at the speed of
/// `input` when it `walks`, else side by side.
///
/// The neck nods the head up by the whole lean of the trunk, so the head looks level, and the
/// view turns it from there as it does standing.
fn deep(input: &AvatarPose, clock: &Clock, walks: bool) -> Pose {
    let speed = input.speed;
    let still = Foot {
        ahead: -HEELS,
        ..Foot::default()
    };
    let mut stance = Stance {
        lift: -SINK.0 - SQUAT,
        lean: HUNCH.0,
        feet: [still, still],
        ..Stance::default()
    };
    if walks {
        stance.lift += 0.5 * bob(clock.phase, speed);
        stance.feet = [
            shortened(foot(clock.phase, speed)),
            shortened(foot(clock.phase + 0.5, speed)),
        ];
    }
    planted(&stance)
        .with(Core::Spine, HUNCH.1)
        .with(Core::HeadPitch, HUNCH.0 + HUNCH.1)
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
