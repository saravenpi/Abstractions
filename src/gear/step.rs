//! One step of the flight gear: mode, spool, attitude, then velocity.

use bevy::prelude::*;

use super::aero::{angle_of_attack, bite, slow};
use super::aim::aim;
use super::boots::push;
use super::input::GearInput;
use super::mode::Mode;
use super::modes::{next, roll, slide};
use super::servo::turn;
use super::state::FlightState;
use super::tune::{MAX_STEP, MAX_SUBSTEPS, THRUST_TIME, WING_TIME};

/// Moves `value` toward 1 when `on`, else toward 0, by `amount`.
fn ramp(value: f32, on: bool, amount: f32) -> f32 {
    match on {
        true => (value + amount).min(1.0),
        false => (value - amount).max(0.0),
    }
}

/// Returns the velocity and the felt load of a body in the air after `h` seconds.
///
/// Boosters and gravity are added first, then the air slows the result implicitly. The load is
/// everything but gravity, over the weight.
fn fly(state: &FlightState, input: &GearInput, h: f32) -> (Vec3, f32) {
    let pushed = push(state, input);
    let free = state.velocity + (pushed - Vec3::Y * input.gravity) * h;
    let slowed = slow(state, free, bite(input), h);
    let felt = pushed + (slowed - free) / h;
    (slowed, felt.length() / input.gravity.max(1e-3))
}

/// Returns the velocity and the felt load for the mode of `state` after `h` seconds.
fn drive(state: &FlightState, input: &GearInput, h: f32) -> (Vec3, f32) {
    match state.mode {
        Mode::Off => (state.velocity, 0.0),
        Mode::Slide => (slide(state.velocity, input.gravity, h), 1.0),
        Mode::Roll => (roll(state.velocity, h), 1.0),
        _ => fly(state, input, h),
    }
}

/// Returns the wind a body in `mode` flies through: the wind of `input` in the air, none on
/// the ground, where the feet hold against it.
fn airflow(mode: Mode, input: &GearInput) -> Vec3 {
    match mode.airborne() && input.wind.is_finite() {
        true => input.wind,
        false => Vec3::ZERO,
    }
}

/// Advances the gear by one substep of `h` seconds.
///
/// The attitude and the air work on the velocity through the air, which is the velocity over
/// the ground less the wind; the mode and the result are over the ground.
fn substep(state: &FlightState, input: &GearInput, h: f32) -> FlightState {
    let mut out = *state;
    out.mode = next(state, input);
    out.servo.clock = if out.mode == state.mode {
        state.servo.clock + h
    } else {
        0.0
    };
    out.servo.impact = 0.0;
    out.wings = ramp(state.wings, out.mode.winged(), h / WING_TIME);
    out.thrust = ramp(
        state.thrust,
        input.boost && out.mode.airborne(),
        h / THRUST_TIME,
    );
    let wind = airflow(out.mode, input);
    out.velocity -= wind;
    let target = aim(&out, input);
    turn(&mut out, &target, h);
    (out.velocity, out.load) = drive(&out, input, h);
    out.airspeed = out.velocity.length();
    out.aoa = angle_of_attack(&out, out.velocity);
    out.velocity += wind;
    out
}

/// Advances the flight gear by `dt` seconds and returns the new state. Pure: no globals, no heap.
///
/// While `state.mode` is [`Mode::Off`] the velocity passes through untouched, so write the
/// controller's own velocity into `state.velocity` before the call; the gear picks it up the
/// instant it takes over. In every other mode the returned velocity is the one to move the body
/// by. Long frames are cut into equal substeps of at most [`MAX_STEP`], which keeps the result
/// the same at any frame rate.
pub fn step(state: &FlightState, input: &GearInput, dt: f32) -> FlightState {
    if dt <= 0.0 || !dt.is_finite() {
        return *state;
    }
    let count = (dt / MAX_STEP).ceil().clamp(1.0, MAX_SUBSTEPS);
    let h = dt / count;
    (0..count as u32).fold(*state, |state, _| substep(&state, input, h))
}
