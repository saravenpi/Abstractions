//! The running state of the animation: time, the leg cycle, the landing crouch, the wing unfold
//! and the pitch of a swimming body.

use std::f32::consts::TAU;

use super::gait::advance;
use super::tune::{CROUCH, CROUCH_RATE, DIP_LIMIT, DIP_RATE, DIP_SPEED};
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

/// Returns how deep the body is asked to crouch: only while landing, deeper for a harder landing.
pub fn crouch_target(input: &AvatarPose) -> f32 {
    match input.mode {
        Mode::Land => CROUCH.0 + (CROUCH.1 - CROUCH.0) * input.landed_hard.clamp(0.0, 1.0),
        _ => 0.0,
    }
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
        let target = crouch_target(input);
        let (offset, vel) = spring(self.crouch - target, self.crouch_vel, CROUCH_RATE, dt);
        (self.crouch, self.crouch_vel) = (offset + target, vel);
        let target = dip_target(input);
        let (offset, rate) = spring(self.dip - target, self.dip_rate, DIP_RATE, dt);
        (self.dip, self.dip_rate) = (offset + target, rate);
        self.still = match input.mode {
            Mode::Idle => self.still + dt,
            _ => 0.0,
        };
        self.unfold = unfold_step(self.unfold, input.wings, dt);
    }
}
