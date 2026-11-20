//! Blending: one weight per mode, eased toward the current mode, and the weighted sum of their poses.
//!
//! Every mode has a pose at every instant, whether it is in use or not, and the weights move by at
//! most `dt` over the blend time per frame. So a change of mode never pops, from any mode to any other,
//! on the ground or in the air.

use bevy::prelude::*;

use super::clock::Clock;
use super::pose::{Core, Pose};
use super::touch::tune::RISE_SECONDS;
use super::tune::BLEND_SECONDS;
use super::{AvatarPose, Mode, air, flight, ground, leap, touch, water};
use crate::player::wrap;

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
    /// The whole change takes the short blend time, or the long one while a sweeping mode is
    /// involved or the body lies `flat` in the arch of a fall. A change into or out of a roll or
    /// a skid takes longer still: the whole body goes down onto the ground and gets up again.
    pub fn step(&mut self, mode: Mode, dt: f32, flat: bool) {
        let held = |of: fn(Mode) -> bool| {
            let mut weights = Mode::ALL.into_iter().zip(self.0);
            weights.any(|(mode, weight)| weight > 0.0 && of(mode))
        };
        let sweeping = mode.sweeping() || flat || held(Mode::sweeping);
        let floored = mode.floored() || held(Mode::floored);
        let rate = dt
            / match (floored, sweeping) {
                (true, _) => RISE_SECONDS,
                (false, true) => BLEND_SECONDS.1,
                (false, false) => BLEND_SECONDS.0,
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
        Mode::Idle => ground::hands(ground::idle(clock), input, clock),
        Mode::Walk | Mode::Run => ground::stride(input, clock),
        Mode::Jump => leap::jump(clock),
        Mode::Fall => air::fall(input, clock),
        Mode::Land => ground::land(input, clock),
        Mode::Swim => water::swim(input, clock),
        Mode::Dive => water::dive(clock),
        Mode::Seated => ground::seated(clock),
        Mode::BootFall => air::boot_fall(input, clock),
        Mode::Glide => flight::glide(input, clock),
        Mode::Fly => flight::fly(input, clock),
        Mode::Roll => touch::roll(input, clock),
        Mode::Skid => touch::skid(input, clock),
    }
}

/// Returns `pose` with the head turned further by `look`.
pub fn looking(pose: Pose, look: Vec2) -> Pose {
    pose.with(Core::HeadYaw, pose.core(Core::HeadYaw) + look.x)
        .with(Core::HeadPitch, pose.core(Core::HeadPitch) + look.y)
}

/// Returns where the view asks the head to look, relative to the trunk: the view of `input`
/// less the pitch the trunk already lies at along a flight path or a way through the water.
///
/// A rolling body keeps its chin in: the view turns its head only as far as it is not rolling.
pub fn asked(weights: &Weights, input: &AvatarPose, clock: &Clock) -> Vec2 {
    let eased = weights.eased();
    let weighed = Mode::ALL.into_iter().zip(eased);
    let wing: f32 = weighed
        .filter(|(mode, _)| mode.winged())
        .map(|(_, w)| w)
        .sum();
    let lain = wing * input.pitch + clock.dip;
    let look = input.look * (1.0 - eased[Mode::Roll as usize]);
    match lain == 0.0 {
        true => look,
        false => look.with_y(wrap(look.y - lain)),
    }
}

/// Returns the pose of a body whose modes weigh `weights`, with the head turned by the look of
/// `input`, relative to the trunk.
///
/// The lean of the winged poses is left free, so the body follows a flight path all the way round
/// in a loop: the pitch of `input` may be on any turn. Every other joint, and the lean of every
/// other pose, stays inside its limits. The pitch of a body along its way through the water comes
/// on top of them: it is eased by itself, so it never adds to the step of a change of mode.
///
/// The whole-body turn of a roll comes on top as well, and as far as the body lies or rolls on
/// the ground its pelvis is moved until the lowest part of it rests there.
pub fn blended(weights: &Weights, input: &AvatarPose, clock: &Clock) -> Pose {
    let (mut pose, mut free, mut floor) = (Pose::default(), 0.0, 0.0);
    for (mode, weight) in Mode::ALL.into_iter().zip(weights.eased()) {
        if weight <= 0.0 {
            continue;
        }
        let part = pose_of(mode, input, clock);
        pose = pose.plus(&part, weight);
        if mode.winged() {
            free += part.core(Core::Pitch) * weight;
        }
        if mode.floored() {
            floor += weight;
        }
    }
    let rest = pose.core(Core::Pitch) - free;
    let held = looking(pose.with(Core::Pitch, rest), input.look).bounded();
    let pitch = held.core(Core::Pitch) + free - clock.dip + clock.turn.x;
    let turned = held
        .with(Core::Pitch, pitch)
        .with(Core::Roll, held.core(Core::Roll) + clock.turn.y);
    touch::grounded(turned, floor)
}
