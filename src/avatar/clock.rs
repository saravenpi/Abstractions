//! The running state of the animation: time, the leg cycle, the landing crouch, the wing unfold
//! and the pitch of a swimming body.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::air::belly;
use super::gait::advance;
use super::touch::{lie, spin};
use super::tune::{
    ARCH_SECONDS, CROUCH, CROUCH_RATE, CROUCH_SOFT, DIP_LIMIT, DIP_RATE, DIP_SPEED, GIVE_SLOW,
    PUSH_OFF, RECOVER_SLOW, RISE_RATE,
};
use super::wing::unfold_step;
use super::{AvatarPose, Mode};

/// Longest distance in meters the leg cycle accepts in one frame, so a teleport does not spin the legs.
const MAX_MOVED: f32 = 1.0;

/// Everything that moves forward with time and that the poses read.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Clock {
    /// Seconds since the body appeared.
    pub time: f64,
    /// Phase of the leg cycle, from 0 to 1.
    pub phase: f32,
    /// The walked distance last seen, in meters.
    pub walked: f32,
    /// Angle of the swim cycle in radians, wrapped at two turns.
    pub stroke: f32,
    /// How far the landing crouch presses the body down, in meters.
    pub crouch: f32,
    /// Speed of the crouch in meters per second.
    pub crouch_vel: f32,
    /// Seconds spent standing still without a break.
    pub still: f32,
    /// How far the wingsuit is unfolded, from 0 to 1.
    pub unfold: f32,
    /// Pitch of the body along its way through the water, positive head up.
    pub dip: f32,
    /// Rate of that pitch in radians per second.
    pub dip_rate: f32,
    /// Seconds since a jump left the ground, held at the end of the push-off in any other air.
    pub air: f32,
    /// Speed along the local up as the air poses read it, in meters per second: the real one,
    /// followed at a bounded rate, so a body that touches down and jumps again in one instant
    /// turns from one pose to the other over a few frames. It holds while the body rolls or skids,
    /// so the legs of the fall it blends out of keep reaching for the ground.
    pub rise: f32,
    /// How far the body lies in the skydiver arch of a fast fall, from 0 to 1: eased toward
    /// what the fall asks, see `air::belly`, by at most [`ARCH_SECONDS`] for the whole way, so
    /// that it never turns over in a frame, whatever mode it leaves or enters.
    pub belly: f32,
    /// Whole-body turn of a roll or a tumble in radians: `x` forward over the head, `y` over the
    /// right side. It counts on past a half turn and drops whole turns once it is still.
    pub turn: Vec2,
    /// Rate of that turn in radians per second.
    pub turn_rate: Vec2,
    /// Distance in meters travelled since the roll began.
    pub rolled: f32,
    /// Distance in meters the roll is expected to carry the body, read as it began.
    pub reach: f32,
    /// Seconds since the roll began.
    pub rolling: f32,
    /// How far the body lies along the ground, from 0 (on its feet) to 1 (on its belly): what it
    /// came in with, held from the touchdown on.
    pub lain: f32,
}

/// Hermite ramp from 0 at `low` to 1 at `high`.
pub fn ramp(low: f32, high: f32, x: f32) -> f32 {
    let t = ((x - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Advances a critically damped spring by `dt` seconds and returns the new offset and speed.
///
/// `offset` is the value minus its target. The step is exact for a fixed target, so it is stable
/// for any `dt`.
pub fn spring(offset: f32, vel: f32, rate: f32, dt: f32) -> (f32, f32) {
    let decay = (-rate * dt).exp();
    let pull = vel + offset * rate;
    (
        (offset + pull * dt) * decay,
        (vel - pull * rate * dt) * decay,
    )
}

/// Returns how deep the body is asked to crouch: only while it takes a landing in, deeper the
/// faster it came down. Under the speeds that make a landing hard the depth still follows the
/// real impact speed, from half the soft depth for a step down to all of it.
pub fn crouch_target(input: &AvatarPose) -> f32 {
    if input.mode != Mode::Land && !input.absorb {
        return 0.0;
    }
    let soft = 0.5 + 0.5 * ramp(CROUCH_SOFT.0, CROUCH_SOFT.1, input.impact);
    CROUCH.0 * soft + (CROUCH.1 - CROUCH.0) * input.landed_hard.clamp(0.0, 1.0)
}

/// Returns the pitch the body lies at in the water: along its way, up to [`DIP_LIMIT`], and less
/// the slower it goes, so a body that only drifts stays upright. Out of the water it is level.
pub fn dip_target(input: &AvatarPose) -> f32 {
    let (ahead, up) = (input.speed.abs(), input.vertical_speed);
    match input.mode.wet() {
        true => {
            let along = up.atan2(ahead).clamp(-DIP_LIMIT, DIP_LIMIT);
            along * ramp(DIP_SPEED.0, DIP_SPEED.1, ahead.hypot(up))
        }
        false => 0.0,
    }
}

impl Clock {
    /// Advances the crouch of the knees by `dt` seconds.
    ///
    /// The knees give way fast under a landing and push the body back up, both more slowly the
    /// harder it was, so the hands of a hard landing reach the ground over a few frames. A jump starts with a short dip: the legs fold as they push off, on the
    /// first frames of the rise, so the real jump is never delayed by it.
    fn absorb(&mut self, input: &AvatarPose, dt: f32) {
        let standing = matches!(input.mode, Mode::Idle | Mode::Walk | Mode::Run | Mode::Land)
            || input.mode.floored();
        self.air = match input.mode {
            _ if standing => 0.0,
            Mode::Jump => self.air + dt,
            _ => PUSH_OFF.1,
        };
        let pushing = input.mode == Mode::Jump && self.air < PUSH_OFF.1;
        let target = if pushing {
            PUSH_OFF.0
        } else {
            crouch_target(input)
        };
        let slow = match target < self.crouch {
            true => RECOVER_SLOW,
            false => GIVE_SLOW,
        };
        let rate = CROUCH_RATE / (1.0 + slow * input.landed_hard.clamp(0.0, 1.0));
        let (offset, vel) = spring(self.crouch - target, self.crouch_vel, rate, dt);
        (self.crouch, self.crouch_vel) = (offset + target, vel);
    }

    /// Returns a slow wave of `rate` radians per second, shifted by `shift` radians.
    pub fn wave(&self, rate: f32, shift: f32) -> f32 {
        (self.time * f64::from(rate) + f64::from(shift)).sin() as f32
    }

    /// Advances everything by `dt` seconds of the state `input` describes.
    pub fn tick(&mut self, input: &AvatarPose, dt: f32) {
        let moved = (input.stride_distance - self.walked).clamp(0.0, MAX_MOVED);
        self.time += f64::from(dt);
        self.phase = advance(self.phase, moved, input.speed);
        self.walked = input.stride_distance;
        self.stroke = (self.stroke + dt * (2.5 + 1.2 * input.speed.abs())).rem_euclid(2.0 * TAU);
        self.absorb(input, dt);
        let climb = RISE_RATE * dt * f32::from(u8::from(!input.mode.floored()));
        self.rise += (input.vertical_speed - self.rise).clamp(-climb, climb);
        let target = dip_target(input);
        let (offset, rate) = spring(self.dip - target, self.dip_rate, DIP_RATE, dt);
        (self.dip, self.dip_rate) = (offset + target, rate);
        self.still = match input.mode {
            Mode::Idle => self.still + dt,
            _ => 0.0,
        };
        self.unfold = unfold_step(self.unfold, input.wings, dt);
        let flat = match input.mode {
            Mode::Fall | Mode::Glide | Mode::Fly => belly(input, self),
            _ => 0.0,
        };
        let turn = dt / ARCH_SECONDS;
        self.belly += (flat - self.belly).clamp(-turn, turn);
        lie(self, input, dt);
        spin(self, input, dt);
    }
}
