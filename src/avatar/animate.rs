//! The animation system: advances the clock and the blend, then writes every segment's transform.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::blend::{Weights, asked, blended};
use super::clock::Clock;
use super::neck::Neck;
use super::pose::Pose;
use super::rig::{SEGS, Seg, locals};
use super::touch::tune::RISE_SECONDS;
use super::{AvatarPose, Mode};
use crate::player::wrap;

/// Longest simulated frame in seconds, to survive hitches.
const MAX_DT: f32 = 0.1;

/// Fastest the flight path pitch, the bank and the load the poses read may change, per second:
/// faster than any flight turns them, so only the step at a touch-down is spread out.
const TRIM_RATE: f32 = 8.0;

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
    /// The state posed last, for what reads the body after it is posed.
    pub input: AvatarPose,
    /// The bank and the felt load the poses read: the real ones, followed at a bounded rate,
    /// because the gear drops both in one step when the body touches down. The bank goes the
    /// short way round, so a body rolling on round is followed through the half turn.
    pub trim: Vec2,
    /// How prone the poses lay the body (`x`) and how far they spread the wings (`y`): the
    /// gear's at once in the air, followed down at the rate of getting up anywhere else, because
    /// the gear stands the body up and folds the wings in a few frames as it touches down while
    /// the flight poses still blend out.
    pub lie: Vec2,
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
        self.input = *input;
        self.clock.tick(input, dt);
        let (none, whole) = winged(&self.weights);
        let leaving = whole && !input.mode.winged();
        let goal = followed(self.path, input.pitch, (none, leaving));
        self.path = if leaving { wrap(self.path) } else { self.path };
        let turn = if none { f32::MAX } else { TRIM_RATE * dt };
        self.path += (goal - self.path).clamp(-turn, turn);
        let want = Vec2::new(self.trim.x + wrap(input.bank - self.trim.x), input.load);
        self.trim += (want - self.trim).clamp_length_max(TRIM_RATE * dt);
        self.trim.x = wrap(self.trim.x);
        let aloft = matches!(
            input.mode,
            Mode::Jump | Mode::Fall | Mode::BootFall | Mode::Glide | Mode::Fly
        );
        let drop = if aloft { f32::MAX } else { dt / RISE_SECONDS };
        self.lie = Vec2::new(input.prone, input.wings).max(self.lie - drop);
        self.weights.step(input.mode, dt, self.clock.belly > 0.0);
        let flown = AvatarPose {
            pitch: self.path,
            bank: self.trim.x,
            load: self.trim.y,
            prone: self.lie.x,
            wings: self.lie.y,
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

/// The segment entities of one body, on its root, in the order of [`Seg::ALL`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skeleton(pub [Entity; SEGS]);

/// Poses every body from its own input: every segment turns in its joint, nothing else moves.
pub fn animate(
    time: Res<Time>,
    mut bodies: Query<(&mut Anim, &AvatarPose, &Skeleton)>,
    mut bones: Query<&mut Transform, With<Bone>>,
) {
    let dt = time.delta_secs().min(MAX_DT);
    for (mut anim, input, skeleton) in &mut bodies {
        let frames = locals(&anim.step(input, dt));
        for (bone, frame) in skeleton.0.into_iter().zip(frames) {
            if let Ok(mut transform) = bones.get_mut(bone) {
                *transform = frame;
            }
        }
    }
}
