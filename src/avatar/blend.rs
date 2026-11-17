//! Blending: one weight per mode, eased toward the current mode, and the weighted sum of their poses.
//!
//! Every mode has a pose at every instant, whether it is in use or not, and the weights move by at
//! most `dt` over the blend time per frame. So a change of mode never pops, from any mode to any other,
//! on the ground or in the air.

use bevy::prelude::*;

use super::clock::Clock;
use super::pose::{Core, Pose};
use super::tune::BLEND_SECONDS;
use super::{AvatarPose, Mode, air, flight, ground, water};

/// Number of modes.
pub const MODES: usize = Mode::ALL.len();

/// How much of each mode is in the pose, in the order of [`Mode::ALL`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weights(pub [f32; MODES]);

impl Default for Weights {
    /// Starts standing still.
    fn default() -> Self {
        Self::only(Mode::Idle)
    }
}

impl Weights {
    /// Returns the weights of a body fully in `mode`.
    pub fn only(mode: Mode) -> Self {
        let mut weights = [0.0; MODES];
        weights[mode as usize] = 1.0;
        Self(weights)
    }

    /// Moves every weight toward a body fully in `mode`, by `dt` seconds.
    ///
    /// The whole change takes the short blend time, or the long one while a sweeping mode is involved.
    pub fn step(&mut self, mode: Mode, dt: f32) {
        let held = Mode::ALL.into_iter().zip(self.0);
        let sweeping = mode.sweeping()
            || held
                .into_iter()
                .any(|(mode, weight)| weight > 0.0 && mode.sweeping());
        let rate = dt
            / if sweeping {
                BLEND_SECONDS.1
            } else {
                BLEND_SECONDS.0
            };
        for (index, weight) in self.0.iter_mut().enumerate() {
            *weight = match index == mode as usize {
                true => (*weight + rate).min(1.0),
                false => (*weight - rate).max(0.0),
            };
        }
    }

    /// Returns the weights eased in and out, summing to one.
    pub fn eased(&self) -> [f32; MODES] {
        let eased = self.0.map(|w| w * w * (3.0 - 2.0 * w));
        let sum: f32 = eased.iter().sum();
        eased.map(|w| w / sum.max(1e-6))
    }
}

/// Returns the pose of `mode` for the state `input` at the instant `clock`.
pub fn pose_of(mode: Mode, input: &AvatarPose, clock: &Clock) -> Pose {
    match mode {
        Mode::Idle => ground::idle(clock),
        Mode::Walk | Mode::Run => ground::stride(input, clock),
        Mode::Jump => air::jump(input),
        Mode::Fall => air::fall(clock),
        Mode::Land => ground::land(clock),
        Mode::Swim => water::swim(input, clock),
        Mode::Dive => water::dive(input, clock),
        Mode::Seated => ground::seated(clock),
        Mode::BootFall => air::boot_fall(input, clock),
        Mode::Glide => flight::glide(input, clock),
        Mode::Fly => flight::fly(input, clock),
    }
}

/// Returns `pose` with the head turned further by `look`.
pub fn looking(pose: Pose, look: Vec2) -> Pose {
    pose.with(Core::HeadYaw, pose.core(Core::HeadYaw) + look.x)
        .with(Core::HeadPitch, pose.core(Core::HeadPitch) + look.y)
}

/// Returns the pose of a body whose modes weigh `weights`, looking where `input` says, with
/// every joint inside its limits.
pub fn blended(weights: &Weights, input: &AvatarPose, clock: &Clock) -> Pose {
    let mut pose = Pose::default();
    for (mode, weight) in Mode::ALL.into_iter().zip(weights.eased()) {
        if weight > 0.0 {
            pose = pose.plus(&pose_of(mode, input, clock), weight);
        }
    }
    looking(pose, input.look).bounded()
}
