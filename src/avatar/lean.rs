//! The whole body on its wings: turned to the attitude the flight gear really holds, and blended
//! with every other pose by the shortest rotation.
//!
//! The trunk joints of a pose are a lean and a roll, and two poses blend by summing them. That
//! is right on the ground, and wrong between a body on its wings and one that falls or lands:
//! the gear may read the same attitude past the vertical with its heading turned round, the
//! roll of the poses is about a level axis where the gear banks about its nose, and summed
//! angles may take the long way round. So while a winged pose holds any weight the trunk is the
//! gear's own attitude, slerped toward the trunk of the other poses: closing the wings, landing
//! or opening them turns the body by at most the angle between where it is and where it goes.

use bevy::prelude::*;

use super::blend::{Weights, blended};
use super::clock::Clock;
use super::pose::{Core, Pose};
use super::{AvatarPose, Mode};

/// Returns the share of `weights` the winged modes hold once eased, and the weights of every
/// other mode alone.
fn split(weights: &Weights) -> (f32, Weights) {
    let eased = Mode::ALL.into_iter().zip(weights.eased());
    let share = eased
        .filter(|(mode, _)| mode.winged())
        .map(|(_, w)| w)
        .sum();
    let mut rest = *weights;
    for (mode, weight) in Mode::ALL.into_iter().zip(rest.0.iter_mut()) {
        if mode.winged() {
            *weight = 0.0;
        }
    }
    (share, rest)
}

/// Returns the turn of the pelvis the lean and the roll of `pose` give, as the rig turns it.
pub fn trunk(pose: &Pose) -> Quat {
    pose.lean
        * Quat::from_rotation_z(-pose.core(Core::Roll))
        * Quat::from_rotation_x(-pose.core(Core::Pitch))
}

/// Returns `pose`, the blend of `weights` for `input`, with its trunk at the attitude of the
/// gear as far as the winged modes weigh, and at the trunk of the other modes for the rest,
/// by the shortest rotation between them. A body with no winged weight, or no attitude in its
/// input, keeps the trunk its joints give.
pub fn lain(pose: Pose, weights: &Weights, input: &AvatarPose, clock: &Clock) -> Pose {
    let Some(attitude) = input.attitude.filter(|turn| turn.is_finite()) else {
        return pose;
    };
    let (share, rest) = split(weights);
    if share <= 0.0 {
        return pose;
    }
    let plain = AvatarPose {
        attitude: None,
        ..*input
    };
    let others = trunk(&blended(&rest, &plain, clock));
    let lean = others
        .slerp(attitude.normalize(), share.min(1.0))
        .normalize();
    Pose { lean, ..pose }
        .with(Core::Pitch, 0.0)
        .with(Core::Roll, 0.0)
}
