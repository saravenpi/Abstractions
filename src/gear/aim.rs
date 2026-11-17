//! Where the attitude wants to go: the look direction leads, the flight path limits.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::aero::{grip, lift_slope};
use super::assist::flare;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::{
    AOA_MAX, AOA_MIN, BANK_GAIN, BANK_SPEED, FOLLOW_SPEED, LIMIT_SPEED, MAX_BANK, MAX_LOAD,
    MAX_PITCH, PITCH_AUTHORITY, PRONE_SPEED, ROLL_AUTHORITY, TRIM_LOAD, TRIM_MAX,
};
use crate::render::smoothstep as smooth;

/// The attitude the gear steers toward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Aim {
    /// Heading of the nose in radians.
    pub(super) yaw: f32,
    /// Nose angle above the horizon in radians.
    pub(super) pitch: f32,
    /// Bank in radians, positive with the right wing down.
    pub(super) bank: f32,
    /// Proneness from 0 to 1.
    pub(super) prone: f32,
    /// Pull of the nose toward the dorsal side in radians.
    pub(super) pull: f32,
}

/// Returns `angle` wrapped to the half turn around zero.
pub(super) fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns the heading of `look` around the local up, or `fallback` when it has none.
pub(super) fn look_yaw(look: Vec3, fallback: f32) -> f32 {
    if look.x * look.x + look.z * look.z < 1e-8 {
        return fallback;
    }
    (-look.x).atan2(-look.z)
}

/// Returns how far the flight path heading lies from `yaw`, zero for a body with no ground speed.
fn path_offset(velocity: Vec3, yaw: f32) -> f32 {
    if velocity.with_y(0.0).length_squared() < 1.0 {
        return 0.0;
    }
    wrap((-velocity.x).atan2(-velocity.z) - yaw)
}

/// Returns the nose-up trim that keeps the flight path on the look direction at this speed and bank.
fn trim(state: &FlightState, input: &GearInput) -> f32 {
    let speed = state.velocity.length();
    let level = state.velocity.with_y(0.0).length() / speed.max(1e-3);
    let load = (level / state.bank.cos().max(0.1)).min(TRIM_LOAD);
    let slope = lift_slope(speed, grip(input.density));
    (input.gravity * load / slope.max(1e-3)).min(TRIM_MAX)
}

/// Returns the largest angle of attack the wing may take at this speed: the stall angle, and
/// less at high speed, where the same angle would pull more than [`MAX_LOAD`] g.
fn lead(state: &FlightState, input: &GearInput) -> f32 {
    let slope = lift_slope(state.velocity.length(), grip(input.density));
    AOA_MAX.min(MAX_LOAD * input.gravity / slope.max(1e-3))
}

/// Returns the pull target: the trim plus the stick, inside what the wing may take.
fn pull(state: &FlightState, input: &GearInput) -> f32 {
    let stick = input.pitch.clamp(-1.0, 1.0) * PITCH_AUTHORITY;
    ((trim(state, input) + stick) * state.spread()).clamp(-AOA_MIN, lead(state, input))
}

/// Returns the pitch target: the look direction and the assist, held near the flight path at speed.
///
/// With the pull, the nose may lead the path by [`lead`] at most. The limits do not wait for
/// the wings: with them folded the flight frame already lies along the fall, so that spreading
/// them never meets the air flat on.
fn pitch(state: &FlightState, input: &GearInput, pull: f32) -> f32 {
    let wanted = input.look.normalize_or_zero().y.asin() + flare(state, input);
    let speed = state.velocity.length();
    let path = (state.velocity.y / speed.max(1e-3)).clamp(-1.0, 1.0).asin();
    let room = (lead(state, input) - pull).max(0.0);
    let held = wanted.clamp(path - AOA_MIN, path + room);
    let weight = smooth(LIMIT_SPEED.0, LIMIT_SPEED.1, speed);
    (wanted + (held - wanted) * weight).clamp(-MAX_PITCH, MAX_PITCH)
}

/// Returns the attitude target for this state and input.
///
/// With the wings folded it is upright along the look heading. As they spread and the airspeed
/// builds, the nose heading follows the flight path, the heading error becomes bank (a
/// coordinated turn, no rudder) and the body lies prone.
pub(super) fn aim(state: &FlightState, input: &GearInput) -> Aim {
    let (spread, speed) = (state.spread(), state.velocity.length());
    let heading = look_yaw(input.look, state.servo.yaw);
    let offset = path_offset(state.velocity, heading);
    let flat = state.velocity.with_y(0.0).length();
    let follow = spread * smooth(FOLLOW_SPEED.0, FOLLOW_SPEED.1, flat);
    let turn = BANK_GAIN * offset + input.roll.clamp(-1.0, 1.0) * ROLL_AUTHORITY;
    let pull = pull(state, input);
    Aim {
        pull,
        yaw: heading + follow * offset,
        pitch: pitch(state, input, pull),
        bank: turn.clamp(-MAX_BANK, MAX_BANK) * spread * smooth(BANK_SPEED.0, BANK_SPEED.1, speed),
        prone: spread * smooth(PRONE_SPEED.0, PRONE_SPEED.1, speed),
    }
}
