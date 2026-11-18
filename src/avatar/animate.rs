//! The animation system: advances the clock and the blend, then writes every segment's transform.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::blend::{Weights, asked, blended};
use super::clock::Clock;
use super::neck::Neck;
use super::pose::Pose;
use super::rig::{Seg, locals};
use super::{Avatar, AvatarPose, Mode};
use crate::player::wrap;

/// Longest simulated frame in seconds, to survive hitches.
const MAX_DT: f32 = 0.1;

/// The animation state of the body.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Anim {
    /// How much of each mode is in the pose.
    pub weights: Weights,
    /// Everything that moves forward with time.
    pub clock: Clock,
    /// The pose applied last.
    pub pose: Pose,
    /// Pitch of the flight path the winged poses lie along: the one asked, on the turn the body
    /// has reached, so that a loop is followed all the way round.
    pub path: f32,
    /// Where the head looks, inside the reach of the neck.
    pub neck: Neck,
}

/// Returns whether the winged modes hold none of the weight of a blend, and whether they hold all of it.
fn winged(weights: &Weights) -> (bool, bool) {
    let held = |winged: bool| {
        let of = Mode::ALL.into_iter().zip(weights.0);
        of.filter(|(mode, _)| mode.winged() == winged)
            .any(|(_, weight)| weight > 0.0)
    };
    (!held(true), !held(false))
}

/// Returns the flight path pitch to pose with, from the one `held` and the one `asked`, which
/// comes wrapped to half a turn around zero.
///
/// The body follows the asked pitch the short way round, so it never jumps when the asked
/// pitch wraps at the top of a loop. Whole turns are dropped only where that shows nothing:
/// with no winged pose in the blend (`none`), and on the frame the body leaves a blend made of
/// winged poses alone (`leaving`), so that it rights itself the short way and ends upright.
pub fn followed(held: f32, asked: f32, (none, leaving): (bool, bool)) -> f32 {
    let held = if leaving { wrap(held) } else { held };
    match none || (asked - held).abs() <= PI {
        true => asked,
        false => held + wrap(asked - held),
    }
}

impl Anim {
    /// Advances the animation by `dt` seconds of the state `input` and returns the new pose.
    pub fn step(&mut self, input: &AvatarPose, dt: f32) -> Pose {
        self.clock.tick(input, dt);
        let (none, whole) = winged(&self.weights);
        let leaving = whole && !input.mode.winged();
        self.path = followed(self.path, input.pitch, (none, leaving));
        self.weights.step(input.mode, dt);
        let flown = AvatarPose {
            pitch: self.path,
            ..*input
        };
        let look = asked(&self.weights, &flown, &self.clock);
        let look = self.neck.step(look, dt);
        self.pose = blended(&self.weights, &AvatarPose { look, ..flown }, &self.clock);
        self.pose
    }
}

/// Marks a segment entity of the body.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bone(pub Seg);

/// Poses the body: every segment turns in its joint, nothing else moves.
pub fn animate(
    time: Res<Time>,
    input: Res<AvatarPose>,
    mut body: Single<&mut Anim, With<Avatar>>,
    mut bones: Query<(&Bone, &mut Transform)>,
) {
    let pose = body.step(&input, time.delta_secs().min(MAX_DT));
    let frames = locals(&pose);
    for (bone, mut transform) in &mut bones {
        *transform = frames[bone.0 as usize];
    }
}
