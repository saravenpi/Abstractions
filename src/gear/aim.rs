//! Where the attitude wants to go: the command of the pilot, held near the flight path at speed.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::aero::{best_aoa, bite, lift_slope};
use super::command::{bank, course};
use super::input::GearInput;
use super::mode::Mode;
use super::path::{climb, path_offset};
use super::state::FlightState;
use super::tune::{
    AOA_MAX, AOA_MIN, BANK_SPEED, FOLLOW_SPEED, LIMIT_SPEED, MAX_LOAD, MAX_PITCH, PATH_LOAD,
    PRONE_SPEED, TRIM_LOAD, TRIM_MAX,
};
use crate::render::smoothstep as smooth;

/// Largest share in radians of the gap between the flight frame and its target that the pull
/// makes up. The frame lags a path that turns, and at speed the wing pulls its whole load with
/// a few hundredths of a radian, about that lag: without this a fast pull-out pulls one g
/// where it may pull four.
const LAG: f32 = 0.1;

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

/// Returns the heading and the flight path angle the body is asked to fly along.
///
/// On the wings it is the command of the pilot, see `command`. In a fall the gear took over it
/// is the flight path, so the folded body lies along its momentum. Otherwise it is the aim the
/// caller gave, or the angles of the look direction.
pub(super) fn asked(state: &FlightState, input: &GearInput) -> Vec2 {
    if state.mode.winged() || state.mode == Mode::Fall {
        return Vec2::new(course(state, input), state.servo.climb);
    }
    let level = look_yaw(input.look, state.servo.yaw);
    input
        .aim
        .unwrap_or_else(|| Vec2::new(level, input.look.normalize_or_zero().y.asin()))
}

/// Returns the largest angle of attack the wing may take at this speed: the stall angle, and
/// less at high speed, where the same angle would pull more than [`MAX_LOAD`] g.
///
/// Gliding with the pitch and roll keys let go it is no more than the angle of the best glide: a
/// hands-off wing too slow for its path drops its nose and gathers speed, where one held at
/// the stall angle would sink on at that angle. Under power the thrust gathers the speed.
fn lead(state: &FlightState, input: &GearInput) -> f32 {
    let slope = lift_slope(state.velocity.length(), bite(input));
    let stall = AOA_MAX.min(MAX_LOAD * input.gravity / slope.max(1e-3));
    let held = input.pitch.abs().max(input.roll.abs()).min(1.0);
    let calm = (1.0 - held) * state.spread() * (1.0 - state.power());
    stall.lerp(stall.min(best_aoa()), calm)
}

/// Returns the pull target: the nose-up trim that carries the weight at this speed and bank,
/// and the pull that turns the flight path onto the asked angle at [`PATH_LOAD`], inside what
/// the wing may take.
fn pull(state: &FlightState, input: &GearInput) -> f32 {
    let speed = state.velocity.length();
    let level = state.velocity.with_y(0.0).length() / speed.max(1e-3);
    let load = (level / state.bank.cos().max(0.1)).min(TRIM_LOAD);
    let turn = wrap(asked(state, input).y - climb(state)).clamp(-1.0, 1.0) * PATH_LOAD;
    let slope = lift_slope(speed, bite(input)).max(1e-3);
    let trim = (input.gravity * load / slope).min(TRIM_MAX) + input.gravity * turn / slope;
    (trim * state.spread()).clamp(-AOA_MIN, lead(state, input))
}

/// Returns the pitch target: the asked angle, held near the flight path at speed, and never
/// past [`MAX_PITCH`], so the frame is never on its back.
///
/// With the pull, the nose may lead the path by [`lead`] at most. The limits do not wait for
/// the wings: with them folded the flight frame already lies along the fall, so that spreading
/// them never meets the air flat on.
fn pitch(state: &FlightState, input: &GearInput, pull: f32) -> f32 {
    let wanted = asked(state, input).y;
    let (speed, path) = (state.velocity.length(), climb(state));
    let room = (lead(state, input) - pull).max(0.0);
    let held = wanted.clamp(path - AOA_MIN, path + room);
    let weight = smooth(LIMIT_SPEED.0, LIMIT_SPEED.1, speed);
    (wanted + (held - wanted) * weight).clamp(-MAX_PITCH, MAX_PITCH)
}

/// Returns the attitude target for this state and input.
///
/// With the wings folded it is upright along the asked heading. As they spread and the airspeed
/// builds, the nose heading follows the flight path, the keys and the view bank the wings, whose
/// banked lift turns the path (a coordinated turn), and the body lies prone. In a fall the gear
/// took over the heading follows the path too, so spreading the wings again turns nothing.
pub(super) fn aim(state: &FlightState, input: &GearInput) -> Aim {
    let (spread, speed) = (state.spread(), state.velocity.length());
    let heading = asked(state, input).x;
    let offset = path_offset(state, heading);
    let flat = state.velocity.with_y(0.0).length();
    let along = spread.max(f32::from(u8::from(state.mode == Mode::Fall)));
    let follow = along * smooth(FOLLOW_SPEED.0, FOLLOW_SPEED.1, flat);
    let pull = pull(state, input);
    let pitch = pitch(state, input, pull);
    Aim {
        pull: pull + (pitch - state.pitch).clamp(-LAG, LAG) * spread,
        yaw: heading + follow * offset,
        pitch,
        bank: bank(state, input) * spread * smooth(BANK_SPEED.0, BANK_SPEED.1, speed),
        prone: spread * smooth(PRONE_SPEED.0, PRONE_SPEED.1, speed),
    }
}
