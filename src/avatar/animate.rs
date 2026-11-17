//! The animation system: advances the clock and the blend, then writes every segment's transform.

use bevy::prelude::*;

use super::blend::{Weights, blended};
use super::clock::Clock;
use super::pose::Pose;
use super::rig::{Seg, locals};
use super::{Avatar, AvatarPose};

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
}

impl Anim {
    /// Advances the animation by `dt` seconds of the state `input` and returns the new pose.
    pub fn step(&mut self, input: &AvatarPose, dt: f32) -> Pose {
        self.clock.tick(input, dt);
        self.weights.step(input.mode, dt);
        self.pose = blended(&self.weights, input, &self.clock);
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
