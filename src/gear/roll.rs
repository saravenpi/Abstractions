//! The roll of the wings: a held key rolls the body on round its nose, and let go the wings
//! come level by themselves.
//!
//! The bank and the pitch are kept within a half turn of zero. A body rolled over and a body
//! over the top of a loop read the same attitude two ways: the bank a half turn over, or the
//! pitch past the vertical with the heading turned round. On the wings the reading is never
//! changed in flight, so nothing that follows the angles jumps. Off them, it is read upright.

use std::f32::consts::PI;

use super::aim::wrap;
use super::command::bank;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::{BANK_RATE, BANK_SPEED, LEVEL, ROLL_RATE};
use crate::render::smoothstep as smooth;

/// Cosines of the pitch between which the wings come level by themselves: not with the nose
/// straight up or down, where no bank is level.
const LEVELS: (f32, f32) = (0.0, 0.3);

/// Cosines of the bank between which the pull turns the pitch at all: with the wings on edge
/// it turns the heading instead.
const EDGE: (f32, f32) = (0.0, 0.3);

/// Returns the nearest bank at which the back of the body faces the sky: zero, or a half turn
/// for a pitch that reads past the vertical.
fn level(state: &FlightState) -> f32 {
    match state.pitch.cos() < 0.0 {
        true => PI.copysign(state.bank),
        false => 0.0,
    }
}

/// Returns which way a pull toward the back of the body turns the pitch: 1 with the wings
/// level, -1 rolled over, through 0 with the wings on edge.
pub(super) fn sense(state: &FlightState) -> f32 {
    let cos = state.bank.cos();
    smooth(EDGE.0, EDGE.1, cos.abs()).copysign(cos)
}

/// Returns how far the back of the body faces the sky, from -1 on its back to 1 the right way up.
pub(super) fn facing(state: &FlightState) -> f32 {
    state.bank.cos() * state.pitch.cos().signum()
}

/// Returns the bank target: a rate, not an attitude.
///
/// A held roll key rolls the wings on at [`ROLL_RATE`] with no limit: the target leads the
/// bank by what the critically damped response at [`BANK_RATE`] needs to roll at that rate.
/// Let go of every key, the wings roll back toward the nearest level bank by [`LEVEL`] of the
/// way, never faster than a held key rolls them, so the body never stays on its back. A held
/// pitch key keeps the bank, so a roll into a bank and a pull flies a turn. While the view
/// steers they bank from level toward it instead, see [`bank`]. Rolls need air: slow, the keys lose their
/// grip and the wings level.
pub(super) fn roll(state: &FlightState, input: &GearInput) -> f32 {
    let speed = state.velocity.length();
    let grip = state.spread() * smooth(BANK_SPEED.0, BANK_SPEED.1, speed);
    let stick = input.roll.clamp(-1.0, 1.0) * grip;
    let rest = level(state) + bank(state, input) * grip;
    let strength = 1.0 - state.spread() * (1.0 - LEVEL);
    let upright = smooth(LEVELS.0, LEVELS.1, state.pitch.cos().abs());
    let pulled = input.pitch.clamp(-1.0, 1.0) * state.spread();
    let free = (1.0 - stick.abs()) * (1.0 - pulled.abs());
    let most = ROLL_RATE * 2.0 / BANK_RATE;
    let back = (wrap(rest - state.bank) * strength * upright * free).clamp(-most, most);
    state.bank + stick * most + back
}

/// Keeps the angles of `state` within a half turn of zero, and reads the body upright while it
/// does not fly on its wings: the same attitude with the heading turned round, the pitch
/// mirrored across the vertical and the bank a half turn over.
///
/// So wings folded after a half loop leave the heading on the way the body really goes, and
/// the fall it folds into is read the right way up: the body is not turned by any of it.
pub(super) fn settle(state: &mut FlightState) {
    if !state.mode.winged() && state.pitch.cos() < 0.0 {
        let pole = PI.copysign(state.pitch);
        state.servo.yaw = wrap(state.servo.yaw + PI);
        (state.pitch, state.servo.climb) = (pole - state.pitch, pole - state.servo.climb);
        state.bank -= PI.copysign(state.bank);
        state.servo.rates.x = -state.servo.rates.x;
    }
    state.pitch = wrap(state.pitch);
    state.bank = wrap(state.bank);
    state.servo.climb = state.pitch + wrap(state.servo.climb - state.pitch);
}
